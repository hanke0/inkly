use axum::extract::{Json, Multipart, Path, Query, State};
use axum::Extension;
use axum::response::IntoResponse;
// (no router helpers here; handlers are wired in `main.rs`)
use serde::Serialize;
use std::sync::Arc;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;

use inkly_contract::dto::{
    BulkIndexIn, CatalogFile, CatalogQuery, CatalogResponse, CatalogSubdir, DocumentDetailResponse,
    DocumentIn, IndexResponse, SearchQuery, SearchResponse, SearchResult, SessionResponse,
};
use std::result::Result;

use inkly_search::SearchError;
use tracing::info;

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

/// Normalizes a logical directory path: `/` or `/segment/.../` (trailing slash except root).
fn normalize_dir_path(raw: &str) -> Result<String, ApiError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok("/".to_string());
    }
    if !s.starts_with('/') {
        return Err(ApiError::bad_request("invalid path"));
    }
    let parts: Vec<&str> = s
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    for p in &parts {
        if *p == ".." {
            return Err(ApiError::bad_request("invalid path"));
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
        return Err(ApiError::bad_request("invalid path"));
    }
    if path == "/" {
        return Ok(());
    }
    if !path.ends_with('/') {
        return Err(ApiError::bad_request("invalid path"));
    }
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        if seg == "." || seg == ".." {
            return Err(ApiError::bad_request("invalid path"));
        }
    }
    Ok(())
}

/// Single tag: non-empty after trim; only Unicode letters, numbers, and `_` (no other punctuation or symbols).
fn validate_tag_format(tag: &str) -> Result<(), ApiError> {
    let t = tag.trim();
    if t.is_empty() {
        return Err(ApiError::bad_request("invalid tags"));
    }
    if t.chars().any(|c| c.is_control()) {
        return Err(ApiError::bad_request("invalid tags"));
    }
    if !t
        .chars()
        .all(|c| c == '_' || c.is_alphanumeric())
    {
        return Err(ApiError::bad_request("invalid tags"));
    }
    Ok(())
}

fn validate_document(input: &DocumentIn) -> Result<(), ApiError> {
    validate_document_path(&input.path)?;
    for t in &input.tags {
        validate_tag_format(t)?;
    }
    Ok(())
}

pub async fn index_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(mut input): Json<DocumentIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    input.path = normalize_dir_path(&input.path)?;
    validate_document(&input)?;

    let tenant_id = user.tenant_id;
    let want_auto_id = use_automatic_doc_id(input.doc_id);
    let requested_doc_id = input.doc_id;
    let title = input.title;
    let content = input.content;
    let doc_url = input.doc_url;
    let tags = input.tags;
    let path = input.path;
    let note = input.note;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        ?requested_doc_id,
        want_auto_id,
        "index_document"
    );

    let (stats, assigned_doc_id) = tokio::task::spawn_blocking(move || {
        let doc_id = if want_auto_id {
            index.allocate_doc_id()?
        } else {
            requested_doc_id.unwrap_or(0)
        };
        let assigned = want_auto_id.then_some(doc_id);
        let stats = index.index_document(
            &tenant_id,
            doc_id,
            &title,
            &content,
            &doc_url,
            &tags,
            &path,
            &note,
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

/// Index a document whose `content` is supplied as a UTF-8 file (`multipart/form-data`, field `file`).
///
/// Other fields match `DocumentIn` as text parts: optional `doc_id` (omit or `0` for server-assigned), `title`, `doc_url`, `path`, `note`, `tags` (comma-separated).
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
        tracing::warn!(error = %e, "multipart field read failed");
        ApiError::bad_request("invalid multipart body")
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                if file_bytes.is_some() {
                    return Err(ApiError::bad_request("duplicate file field"));
                }
                let bytes = field.bytes().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart file bytes read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            "doc_id" => {
                if doc_id_raw.is_some() {
                    return Err(ApiError::bad_request("duplicate doc_id field"));
                }
                let t = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart doc_id read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
                doc_id_raw = Some(t);
            }
            "title" => {
                title = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart title read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
            }
            "doc_url" => {
                doc_url = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart doc_url read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
            }
            "path" => {
                path = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart path read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
            }
            "note" => {
                note = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart note read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
            }
            "tags" => {
                tags_raw = field.text().await.map_err(|e| {
                    tracing::warn!(error = %e, "multipart tags read failed");
                    ApiError::bad_request("invalid multipart body")
                })?;
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| ApiError::bad_request("missing file"))?;
    let content = String::from_utf8(bytes).map_err(|_| ApiError::bad_request("file must be utf-8"))?;

    let doc_id = match doc_id_raw.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(s) => {
            let n = s
                .parse::<u64>()
                .map_err(|_| ApiError::bad_request("invalid doc_id"))?;
            Some(n)
        }
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

    let tenant_id = user.tenant_id;
    let want_auto_id = use_automatic_doc_id(input.doc_id);
    let requested_doc_id = input.doc_id;
    let title = input.title;
    let content = input.content;
    let doc_url = input.doc_url;
    let tags = input.tags;
    let path = input.path;
    let note = input.note;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        ?requested_doc_id,
        want_auto_id,
        "index_document_upload"
    );

    let (stats, assigned_doc_id) = tokio::task::spawn_blocking(move || {
        let doc_id = if want_auto_id {
            index.allocate_doc_id()?
        } else {
            requested_doc_id.unwrap_or(0)
        };
        let assigned = want_auto_id.then_some(doc_id);
        let stats = index.index_document(
            &tenant_id,
            doc_id,
            &title,
            &content,
            &doc_url,
            &tags,
            &path,
            &note,
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

pub async fn index_documents_bulk(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(mut input): Json<BulkIndexIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    const MAX_BATCH: usize = 200;
    if input.documents.is_empty() || input.documents.len() > MAX_BATCH {
        return Err(ApiError::bad_request("invalid documents batch size"));
    }
    for doc in &mut input.documents {
        doc.path = normalize_dir_path(&doc.path)?;
        validate_document(doc)?;
    }

    let tenant_id = user.tenant_id;
    let documents = input.documents;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        indexed_documents = documents.len(),
        "index_documents_bulk"
    );

    let (stats, doc_ids) = tokio::task::spawn_blocking(move || {
        let mut rows: Vec<(u64, String, String, String, Vec<String>, String, String)> =
            Vec::with_capacity(documents.len());
        let mut ids = Vec::with_capacity(documents.len());
        for d in documents {
            let doc_id = if use_automatic_doc_id(d.doc_id) {
                index.allocate_doc_id()?
            } else {
                d.doc_id.unwrap_or(0)
            };
            ids.push(doc_id);
            rows.push((
                doc_id,
                d.title,
                d.content,
                d.doc_url,
                d.tags,
                d.path,
                d.note,
            ));
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

pub async fn search(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let tenant_id = user.tenant_id;
    let q = query.q;
    let limit = query.limit;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        query = %q,
        limit = limit,
        "search"
    );

    let (total_hits, items) = tokio::task::spawn_blocking(move || index.search(&tenant_id, &q, limit))
        .await
        .map_err(|_| ApiError::Internal)??;

    let results = items
        .into_iter()
        .map(|it| SearchResult {
            doc_id: it.doc_id,
            title: it.title,
            doc_url: it.doc_url,
            snippet: it.snippet,
            score: it.score,
            created_at: it.created_at,
            updated_at: it.updated_at,
            tags: it.tags,
            path: it.path,
            note: it.note,
        })
        .collect();

    Ok(Json(SearchResponse {
        total_hits,
        results,
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
        return Err(ApiError::bad_request("invalid doc_id"));
    }

    let tenant_id = user.tenant_id;
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        doc_id = doc_id,
        "get_document"
    );

    let doc = tokio::task::spawn_blocking(move || index.get_document(&tenant_id, doc_id))
        .await
        .map_err(|_| ApiError::Internal)??;

    let d = doc.ok_or(ApiError::NotFound)?;

    Ok(Json(DocumentDetailResponse {
        doc_id: d.doc_id,
        title: d.title,
        content: d.content,
        doc_url: d.doc_url,
        path: d.path,
        note: d.note,
        tags: d.tags,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }))
}

