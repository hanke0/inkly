use axum::extract::{Json, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::response::IntoResponse;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::result::Result;

use crate::auth::AuthUser;
use crate::error::{user_message_for_search_query_error, ApiError};
use crate::state::AppState;

use inkly_contract::dto::{
    BulkIndexIn, CatalogFile, CatalogQuery, CatalogResponse, CatalogSubdir, DocumentDetailResponse,
    DocumentIn, IndexResponse, SearchQuery, SearchResponse, SearchResult, SessionResponse,
};
use inkly_search::{DocumentRow, SearchError, SearchResultItem, StoredDocument};
use tracing::{info, warn};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn healthz() -> impl IntoResponse {
    axum::Json(HealthResponse { status: "ok" })
}

/// Validates `Authorization: Basic` and returns 200 when credentials match the server config.
pub async fn session(Extension(user): Extension<AuthUser>) -> Result<Json<SessionResponse>, ApiError> {
    info!(
        user_id = %user.user_id,
        tenant_id = %user.tenant_id,
        "session"
    );
    Ok(Json(SessionResponse { ok: true }))
}

// ---------------------------------------------------------------------------
// Input validation helpers
// ---------------------------------------------------------------------------

/// Normalizes a logical directory path: `/` or `/segment/.../` (trailing slash except root).
fn normalize_dir_path(raw: &str) -> Result<String, ApiError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok("/".to_string());
    }
    if !s.starts_with('/') {
        return Err(ApiError::bad_request(
            "Path must start with `/` (for example `/notes/`). Fix the path and try again.",
        ));
    }
    let parts: Vec<&str> = s
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    for p in &parts {
        if *p == ".." {
            return Err(ApiError::bad_request(
                "Path cannot contain `..` segments. Use a folder path under your workspace.",
            ));
        }
    }
    if parts.is_empty() {
        return Ok("/".to_string());
    }
    Ok(format!("/{}/", parts.join("/")))
}

fn use_automatic_doc_id(doc_id: Option<u64>) -> bool {
    matches!(doc_id, None | Some(0))
}

/// Logical path after `normalize_dir_path`: `/` or `/segment/.../` (no `.` / `..` segments).
fn validate_document_path(path: &str) -> Result<(), ApiError> {
    if path.is_empty() || !path.starts_with('/') {
        return Err(ApiError::bad_request(
            "Path must start with `/` (for example `/notes/`). Fix the path and try again.",
        ));
    }
    if path == "/" {
        return Ok(());
    }
    if !path.ends_with('/') {
        return Err(ApiError::bad_request(
            "Document path must end with `/` because it names a folder (for example `/notes/`).",
        ));
    }
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        if seg == "." || seg == ".." {
            return Err(ApiError::bad_request(
                "Path cannot contain `.` or `..` segments. Use a normal folder name.",
            ));
        }
    }
    Ok(())
}

/// Single tag: non-empty after trim; only Unicode letters, numbers, and `_`.
fn validate_tag_format(tag: &str) -> Result<(), ApiError> {
    let t = tag.trim();
    if t.is_empty() {
        return Err(ApiError::bad_request(
            "Each tag must be non-empty after trimming spaces.",
        ));
    }
    if t.chars().any(|c| c.is_control()) {
        return Err(ApiError::bad_request(
            "Tags cannot contain control characters. Use letters, numbers, and underscores.",
        ));
    }
    if !t.chars().all(|c| c == '_' || c.is_alphanumeric()) {
        return Err(ApiError::bad_request(
            "Tags may only contain letters, numbers, and underscores.",
        ));
    }
    Ok(())
}

fn validate_document(input: &DocumentIn) -> Result<(), ApiError> {
    if input.title.trim().is_empty() {
        return Err(ApiError::bad_request(
            "Title is required. Enter a non-empty title and try again.",
        ));
    }
    validate_document_path(&input.path)?;
    for t in &input.tags {
        validate_tag_format(t)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn into_search_result(it: SearchResultItem) -> SearchResult {
    SearchResult {
        doc_id: it.doc_id,
        title: it.title,
        doc_url: it.doc_url,
        snippet: it.snippet,
        summary: it.summary,
        score: it.score,
        created_at: it.created_at,
        updated_at: it.updated_at,
        tags: it.tags,
        path: it.path,
        note: it.note,
    }
}

fn into_document_detail(d: StoredDocument) -> DocumentDetailResponse {
    DocumentDetailResponse {
        doc_id: d.doc_id,
        title: d.title,
        content: d.content,
        summary: d.summary,
        doc_url: d.doc_url,
        path: d.path,
        note: d.note,
        tags: d.tags,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Multipart helpers
// ---------------------------------------------------------------------------

async fn multipart_text(field: axum::extract::multipart::Field<'_>, name: &str) -> Result<String, ApiError> {
    field.text().await.map_err(|e| {
        warn!(error = %e, name, "multipart text read failed");
        ApiError::bad_request(
            "Could not read the multipart form. Retry the upload and ensure the request is multipart/form-data.",
        )
    })
}

// ---------------------------------------------------------------------------
// Summarization
// ---------------------------------------------------------------------------

/// When summarization is off or the lock fails, returns an empty string (indexing still succeeds).
fn summarize_if_enabled(
    summarizer: &Option<Arc<Mutex<inkly_summarize::Summarizer>>>,
    content: &str,
    op: &'static str,
) -> String {
    let Some(sm) = summarizer else {
        return String::new();
    };
    let mut summary = String::new();
    match sm.lock() {
        Ok(mut guard) => {
            let t = std::time::Instant::now();
            match guard.summarize(content) {
                Ok(s) => {
                    let elapsed = t.elapsed();
                    tracing::info!(
                        op,
                        elapsed_ms = elapsed.as_millis(),
                        summary_chars = s.len(),
                        "summarize completed"
                    );
                    summary = s;
                }
                Err(e) => warn!(error = %e, op, "summarizer failed"),
            }
        }
        Err(_) => warn!(op, "summarizer lock poisoned"),
    }
    summary
}

// ---------------------------------------------------------------------------
// Shared indexing logic
// ---------------------------------------------------------------------------

/// Core blocking work shared by `index_document` and `index_document_upload`.
/// Callers are responsible for normalizing and validating `input` before calling this.
async fn perform_index_document(
    state: Arc<AppState>,
    user: AuthUser,
    input: DocumentIn,
    op: &'static str,
) -> Result<Json<IndexResponse>, ApiError> {
    let tenant_id = user.tenant_id;
    let want_auto_id = use_automatic_doc_id(input.doc_id);
    let requested_doc_id = input.doc_id;

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        ?requested_doc_id,
        want_auto_id,
        op,
        "index_document"
    );

    let index = state.index.clone();
    let summarizer = state.summarizer.clone();

    let (content, existing_summary) = if want_auto_id {
        let c = input.content.ok_or_else(|| {
            ApiError::bad_request(
                "New documents require body content. Paste or type content in the editor, then save.",
            )
        })?;
        if c.trim().is_empty() {
            return Err(ApiError::bad_request(
                "Content cannot be empty. Add text or HTML before saving.",
            ));
        }
        (c, None)
    } else {
        let existing_doc_id = requested_doc_id.unwrap_or(0);
        let idx = index.clone();
        let tid = tenant_id.clone();
        let existing = tokio::task::spawn_blocking(move || idx.get_document(&tid, existing_doc_id))
            .await
            .map_err(|_| ApiError::Internal)??
            .ok_or(ApiError::NotFound)?;
        (existing.content, Some(existing.summary))
    };

    let doc = DocumentRow {
        doc_id: 0,
        title: input.title,
        content: content.clone(),
        doc_url: input.doc_url,
        summary: String::new(),
        tags: input.tags,
        path: input.path,
        note: input.note,
    };

    let (stats, assigned_doc_id) = tokio::task::spawn_blocking(move || {
        let doc_id = if want_auto_id {
            index.allocate_doc_id()?
        } else {
            requested_doc_id.unwrap_or(0)
        };
        let assigned = want_auto_id.then_some(doc_id);
        let summary = existing_summary
            .unwrap_or_else(|| summarize_if_enabled(&summarizer, &content, op));
        let stats = index.index_document(
            &tenant_id,
            DocumentRow {
                doc_id,
                summary,
                ..doc
            },
        )?;
        Ok::<_, SearchError>((stats, assigned))
    })
    .await
    .map_err(|_| ApiError::Internal)??;

    Ok(Json(IndexResponse {
        indexed: stats.indexed,
        deleted: stats.deleted,
        doc_id: assigned_doc_id,
        doc_ids: Vec::new(),
    }))
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

pub async fn index_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(mut input): Json<DocumentIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    input.path = normalize_dir_path(&input.path)?;
    validate_document(&input)?;
    perform_index_document(state, user, input, "index_document").await
}

/// Index a document whose `content` is supplied as a UTF-8 file (`multipart/form-data`, field `file`).
///
/// Other fields match `DocumentIn` as text parts: optional `doc_id` (omit or `0` for server-assigned),
/// `title`, `doc_url`, `path`, `note`, `tags` (comma-separated).
pub async fn index_document_upload(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<IndexResponse>, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut doc_id_raw: Option<String> = None;
    let mut title = String::new();
    let mut doc_url = String::new();
    let mut path = String::new();
    let mut note = String::new();
    let mut tags_raw = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        warn!(error = %e, "multipart field read failed");
        ApiError::bad_request(
            "Could not read the multipart form. Retry the upload and ensure the request is multipart/form-data.",
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                if file_bytes.is_some() {
                    return Err(ApiError::bad_request(
                        "The form contains more than one `file` field. Send a single file part.",
                    ));
                }
                let bytes = field.bytes().await.map_err(|e| {
                    warn!(error = %e, "multipart file bytes read failed");
                    ApiError::bad_request(
                        "Could not read the multipart form. Retry the upload and ensure the request is multipart/form-data.",
                    )
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            "doc_id" => {
                if doc_id_raw.is_some() {
                    return Err(ApiError::bad_request(
                        "The form contains more than one `doc_id` field. Send a single doc_id value.",
                    ));
                }
                doc_id_raw = Some(multipart_text(field, "doc_id").await?);
            }
            "title" => title = multipart_text(field, "title").await?,
            "doc_url" => doc_url = multipart_text(field, "doc_url").await?,
            "path" => path = multipart_text(field, "path").await?,
            "note" => note = multipart_text(field, "note").await?,
            "tags" => tags_raw = multipart_text(field, "tags").await?,
            _ => {}
        }
    }

    let doc_id = match doc_id_raw.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(s) => {
            let n = s
                .parse::<u64>()
                .map_err(|_| {
                    ApiError::bad_request(
                        "`doc_id` must be a non-negative whole number (or omit / use 0 for a new document).",
                    )
                })?;
            Some(n)
        }
    };

    let content = if use_automatic_doc_id(doc_id) {
        let bytes = file_bytes.ok_or_else(|| {
            ApiError::bad_request(
                "New documents need a `file` part in the multipart body. Add the file field and try again.",
            )
        })?;
        Some(String::from_utf8(bytes).map_err(|_| {
            ApiError::bad_request(
                "The uploaded file must be valid UTF-8 text or HTML. Convert the encoding and try again.",
            )
        })?)
    } else {
        None
    };

    let tags: Vec<String> = tags_raw
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    let mut input = DocumentIn {
        doc_id,
        title: title.trim().to_string(),
        content,
        doc_url: doc_url.trim().to_string(),
        tags,
        path: path.trim().to_string(),
        note,
    };

    input.path = normalize_dir_path(&input.path)?;
    validate_document(&input)?;

    perform_index_document(state, user, input, "index_document_upload").await
}

pub async fn index_documents_bulk(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(mut input): Json<BulkIndexIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    const MAX_BATCH: usize = 200;
    if input.documents.is_empty() || input.documents.len() > MAX_BATCH {
        return Err(ApiError::bad_request(
            "Bulk index expects between 1 and 200 documents in one request. Reduce or split the batch and try again.",
        ));
    }
    for doc in &mut input.documents {
        doc.path = normalize_dir_path(&doc.path)?;
        validate_document(doc)?;
    }

    let tenant_id = user.tenant_id;
    let documents = input.documents;
    let index = state.index.clone();
    let summarizer = state.summarizer.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        count = documents.len(),
        "index_documents_bulk"
    );

    let (stats, doc_ids) = tokio::task::spawn_blocking(move || {
        let mut rows: Vec<DocumentRow> = Vec::with_capacity(documents.len());
        let mut ids: Vec<u64> = Vec::with_capacity(documents.len());
        for d in documents {
            let want_auto = use_automatic_doc_id(d.doc_id);
            let doc_id = if want_auto {
                index.allocate_doc_id()?
            } else {
                d.doc_id.unwrap_or(0)
            };
            let (content, existing_summary) = if want_auto {
                let c = d.content.unwrap_or_default();
                if c.trim().is_empty() {
                    return Err(SearchError::InvalidInput(
                        "content must not be empty".into(),
                    ));
                }
                (c, None)
            } else {
                let existing = index.get_document(&tenant_id, doc_id)?;
                match existing {
                    Some(e) => (e.content, Some(e.summary)),
                    None => (String::new(), None),
                }
            };
            ids.push(doc_id);
            let summary = existing_summary
                .unwrap_or_else(|| summarize_if_enabled(&summarizer, &content, "index_documents_bulk"));
            rows.push(DocumentRow {
                doc_id,
                title: d.title,
                content,
                doc_url: d.doc_url,
                summary,
                tags: d.tags,
                path: d.path,
                note: d.note,
            });
        }
        let stats = index.index_documents(&tenant_id, rows)?;
        Ok::<_, SearchError>((stats, ids))
    })
    .await
    .map_err(|_| ApiError::Internal)??;

    Ok(Json(IndexResponse {
        indexed: stats.indexed,
        deleted: stats.deleted,
        doc_id: None,
        doc_ids,
    }))
}

const MAX_SEARCH_TAG_FILTERS: usize = 20;

pub async fn search(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let tenant_id = user.tenant_id;
    let q = query.q;
    let limit = query.limit;
    let index = state.index.clone();

    let path_filter = match query.path.as_deref() {
        None | Some("") => None,
        Some(p) => Some(normalize_dir_path(p)?),
    };
    let path_filter = path_filter.filter(|p| p != "/");

    let tag_filters: Vec<String> = query
        .tags
        .as_deref()
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if tag_filters.len() > MAX_SEARCH_TAG_FILTERS {
        return Err(ApiError::bad_request(
            "Too many tag filters in this search. Remove some tag filters (comma-separated) and try again.",
        ));
    }
    for t in &tag_filters {
        validate_tag_format(t)?;
    }

    let q_trimmed = q.trim().to_string();
    if q_trimmed.is_empty() && path_filter.is_none() && tag_filters.is_empty() {
        return Err(ApiError::bad_request(
            "Enter search text and/or pick a folder path or tag filter. At least one of these is required.",
        ));
    }

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        query = %q_trimmed,
        limit,
        path = ?path_filter,
        tag_filters = ?tag_filters,
        "search"
    );

    let (total_hits, items) = tokio::task::spawn_blocking(move || {
        index.search(
            &tenant_id,
            &q_trimmed,
            limit,
            path_filter.as_deref(),
            &tag_filters,
        )
    })
    .await
    .map_err(|_| ApiError::Internal)?
    .map_err(|e| match e {
        SearchError::InvalidInput(msg) => {
            ApiError::bad_request(user_message_for_search_query_error(&msg))
        }
        _ => ApiError::Internal,
    })?;

    Ok(Json(SearchResponse {
        total_hits,
        results: items.into_iter().map(into_search_result).collect(),
    }))
}

pub async fn catalog(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<CatalogResponse>, ApiError> {
    let dir_path = normalize_dir_path(&query.path)?;
    let tenant_id = user.tenant_id;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        path = %dir_path,
        "catalog"
    );

    let listing = tokio::task::spawn_blocking(move || index.catalog_list(&tenant_id, &dir_path))
        .await
        .map_err(|_| ApiError::Internal)??;

    Ok(Json(CatalogResponse {
        path: listing.path,
        subdirs: listing
            .subdirs
            .into_iter()
            .map(|(name, path)| CatalogSubdir { name, path })
            .collect(),
        files: listing
            .files
            .into_iter()
            .map(|(doc_id, title)| CatalogFile { doc_id, title })
            .collect(),
    }))
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Path(doc_id): Path<u64>,
) -> Result<Json<DocumentDetailResponse>, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(
            "Document ID must be a positive number. Use the ID shown in search or the catalog.",
        ));
    }

    let tenant_id = user.tenant_id;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        doc_id,
        "get_document"
    );

    let doc = tokio::task::spawn_blocking(move || index.get_document(&tenant_id, doc_id))
        .await
        .map_err(|_| ApiError::Internal)??;

    let d = doc.ok_or(ApiError::NotFound)?;
    Ok(Json(into_document_detail(d)))
}

pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Path(doc_id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(
            "Document ID must be a positive number. Use the ID shown in search or the catalog.",
        ));
    }

    let tenant_id = user.tenant_id;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        doc_id,
        "delete_document"
    );

    let removed = tokio::task::spawn_blocking(move || index.delete_document(&tenant_id, doc_id))
        .await
        .map_err(|_| ApiError::Internal)?
        .map_err(ApiError::from)?;

    if !removed {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
