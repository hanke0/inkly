use axum::Extension;
use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use std::result::Result;
use std::sync::Arc;

use crate::auth::AuthUser;
use crate::error::{ApiError, map_index_error, map_search_error};
use crate::i18n::{Msg, t};
use crate::locale::Locale;
use crate::state::AppState;
use crate::summary_queue::EnqueueOutcome;

use inkly_contract::dto::{
    CatalogFile, CatalogQuery, CatalogResponse, CatalogSubdir, DocumentDetailResponse, DocumentIn,
    IndexResponse, SearchQuery, SearchResponse, SearchResult, SessionResponse,
    SummaryEnqueueResponse,
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
pub async fn session(
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
) -> Result<Json<SessionResponse>, ApiError> {
    info!(
        user_id = %user.user_id,
        tenant_id = %user.tenant_id,
        "session"
    );
    Ok(Json(SessionResponse {
        ok: true,
        locale: locale.as_api_tag().to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Input validation helpers
// ---------------------------------------------------------------------------

/// Normalizes a logical directory path to `/` or `/segment/.../`.
///
/// Guarantees: leading `/`; non-root paths end with `/`; empty and `.` segments removed; `..` rejected.
/// Callers should use this as the only path canonicalization before persisting or comparing paths.
fn normalize_dir_path(raw: &str, locale: Locale) -> Result<String, ApiError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok("/".to_string());
    }
    if !s.starts_with('/') {
        return Err(ApiError::bad_request(t(locale, Msg::InvalidFolderPath)));
    }
    let parts: Vec<&str> = s
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    for p in &parts {
        if *p == ".." {
            return Err(ApiError::bad_request(t(locale, Msg::InvalidFolderPath)));
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

/// Single tag: non-empty after trim; only Unicode letters, numbers, and `_`.
fn validate_tag_format(tag: &str, locale: Locale) -> Result<(), ApiError> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request(t(locale, Msg::TagNonempty)));
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err(ApiError::bad_request(t(locale, Msg::TagNoControl)));
    }
    if !trimmed.chars().all(|c| c == '_' || c.is_alphanumeric()) {
        return Err(ApiError::bad_request(t(locale, Msg::TagAlphanumeric)));
    }
    Ok(())
}

fn validate_document(input: &DocumentIn, locale: Locale) -> Result<(), ApiError> {
    if input.title.trim().is_empty() {
        return Err(ApiError::bad_request(t(locale, Msg::TitleRequired)));
    }
    for t in &input.tags {
        validate_tag_format(t, locale)?;
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

async fn multipart_text(
    field: axum::extract::multipart::Field<'_>,
    name: &str,
    locale: Locale,
) -> Result<String, ApiError> {
    field.text().await.map_err(|e| {
        warn!(error = %e, name, "multipart text read failed");
        ApiError::bad_request(t(locale, Msg::MultipartReadFailed))
    })
}

// ---------------------------------------------------------------------------
// Shared indexing logic
// ---------------------------------------------------------------------------

/// Core blocking work shared by `index_document_upload` and `update_document`.
/// Callers are responsible for normalizing and validating `input` before calling this.
async fn perform_index_document(
    state: Arc<AppState>,
    user: AuthUser,
    input: DocumentIn,
    existing_summary: Option<String>,
    locale: Locale,
) -> Result<Json<IndexResponse>, ApiError> {
    let tenant_id = user.tenant_id;
    let want_auto_id = use_automatic_doc_id(input.doc_id);
    let requested_doc_id = input.doc_id;

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        ?requested_doc_id,
        want_auto_id,
        "index_document"
    );

    let index = state.index.clone();
    let defer_summary_to_queue = want_auto_id && state.summary_queue.is_some();

    if input.content.trim().is_empty() {
        return Err(ApiError::bad_request(t(locale, Msg::BulkContentEmpty)));
    }
    let content = input.content;

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

    let tenant_for_index = tenant_id.clone();
    let (stats, assigned_doc_id) = tokio::task::spawn_blocking(move || {
        let doc_id = if want_auto_id {
            index.allocate_doc_id()?
        } else {
            requested_doc_id.unwrap_or(0)
        };
        let assigned = want_auto_id.then_some(doc_id);
        let summary = if defer_summary_to_queue {
            String::new()
        } else {
            existing_summary.unwrap_or_default()
        };
        let stats = index.index_document(
            &tenant_for_index,
            DocumentRow {
                doc_id,
                summary,
                ..doc
            },
        )?;
        Ok::<_, SearchError>((stats, assigned))
    })
    .await
    .map_err(|_| ApiError::internal(locale))?
    .map_err(|e| map_index_error(e, locale))?;

    if defer_summary_to_queue {
        if let Some(q) = state.summary_queue.as_ref() {
            let Some(doc_id) = assigned_doc_id else {
                warn!(tenant_id = %tenant_id, "auto id missing after index; skip summary enqueue");
                return Ok(Json(IndexResponse {
                    indexed: stats.indexed,
                    deleted: stats.deleted,
                    doc_id: assigned_doc_id,
                    doc_ids: Vec::new(),
                }));
            };
            match q.enqueue(&tenant_id, doc_id) {
                Ok(EnqueueOutcome::Enqueued) => {
                    info!(tenant_id = %tenant_id, doc_id, "summary job enqueued");
                }
                Ok(EnqueueOutcome::AlreadyQueued) => {
                    warn!(tenant_id = %tenant_id, doc_id, "summary already queued after new doc");
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        tenant_id = %tenant_id,
                        doc_id,
                        "summary enqueue failed; document saved without queued summary"
                    );
                }
            }
        } else {
            warn!(
                tenant_id = %tenant_id,
                "summarizer enabled but summary queue missing; document saved without queued summary"
            );
        }
    }

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

async fn parse_document_multipart(
    mut multipart: Multipart,
    require_file: bool,
    locale: Locale,
) -> Result<(DocumentIn, bool), ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut title = String::new();
    let mut doc_url = String::new();
    let mut path = String::new();
    let mut note = String::new();
    let mut tags_raw = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        warn!(error = %e, "multipart field read failed");
        ApiError::bad_request(t(locale, Msg::MultipartReadFailed))
    })? {
        let name = field.name().unwrap_or("");
        match name {
            "file" => {
                if file_bytes.is_some() {
                    return Err(ApiError::bad_request(t(locale, Msg::MultipartMultipleFile)));
                }
                let bytes = field.bytes().await.map_err(|e| {
                    warn!(error = %e, "multipart file bytes read failed");
                    ApiError::bad_request(t(locale, Msg::MultipartReadFailed))
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            "title" => title = multipart_text(field, "title", locale).await?,
            "doc_url" => doc_url = multipart_text(field, "doc_url", locale).await?,
            "path" => path = multipart_text(field, "path", locale).await?,
            "note" => note = multipart_text(field, "note", locale).await?,
            "tags" => tags_raw = multipart_text(field, "tags", locale).await?,
            _ => {}
        }
    }

    let content = match file_bytes {
        Some(bytes) => (
            String::from_utf8(bytes)
                .map_err(|_| ApiError::bad_request(t(locale, Msg::UploadedFileUtf8)))?,
            true,
        ),
        None if require_file => {
            return Err(ApiError::bad_request(t(locale, Msg::NewDocNeedsFilePart)));
        }
        None => (String::new(), false),
    };

    let tags: Vec<String> = tags_raw
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    Ok((
        DocumentIn {
            doc_id: None,
            title: title.trim().to_string(),
            content: content.0,
            doc_url: doc_url.trim().to_string(),
            tags,
            path: path.trim().to_string(),
            note,
        },
        content.1,
    ))
}

/// Create a new document: `content` as a UTF-8 file (`multipart/form-data`, field `file`).
///
/// `POST /v1/documents`. Text parts: `title`, `doc_url`, `path`, `note`, `tags` (comma-separated).
/// The server always assigns a new `doc_id` (any extra parts such as `doc_id` are ignored).
pub async fn index_document_upload(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    multipart: Multipart,
) -> Result<Json<IndexResponse>, ApiError> {
    let (mut input, _has_file) = parse_document_multipart(multipart, true, locale).await?;

    input.path = normalize_dir_path(&input.path, locale)?;
    validate_document(&input, locale)?;

    perform_index_document(state, user, input, None, locale).await
}

/// Update document fields via multipart form-data.
///
/// `POST /v1/documents/{doc_id}`. `doc_id` from path is authoritative.
/// Text parts: `title`, `doc_url`, `path`, `note`, `tags` (comma-separated). Optional `file` replaces content.
pub async fn update_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    Path(doc_id): Path<u64>,
    multipart: Multipart,
) -> Result<Json<IndexResponse>, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(t(locale, Msg::DocIdPositive)));
    }
    let (mut input, has_file) = parse_document_multipart(multipart, false, locale).await?;
    let tenant_id = user.tenant_id.clone();
    let index = state.index.clone();
    let existing = tokio::task::spawn_blocking(move || index.get_document(&tenant_id, doc_id))
        .await
        .map_err(|_| ApiError::internal(locale))?
        .map_err(|e| map_index_error(e, locale))?
        .ok_or_else(|| ApiError::not_found(locale))?;
    if !has_file {
        input.content = existing.content.clone();
    }
    input.doc_id = Some(doc_id);
    input.path = normalize_dir_path(&input.path, locale)?;
    validate_document(&input, locale)?;
    perform_index_document(state, user, input, Some(existing.summary), locale).await
}

/// Queue async regeneration of the document summary (or report that it is already queued).
///
/// `POST /v1/documents/{doc_id}/summary`. Requires summarization to be enabled on the server.
pub async fn enqueue_document_summary(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    Path(doc_id): Path<u64>,
) -> Result<Json<SummaryEnqueueResponse>, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(t(locale, Msg::DocIdPositive)));
    }

    let Some(ref queue) = state.summary_queue else {
        return Err(ApiError::bad_request(t(locale, Msg::SummarizeDisabled)));
    };

    let tenant_id = user.tenant_id.clone();
    let index = state.index.clone();

    let exists = tokio::task::spawn_blocking(move || index.get_document(&tenant_id, doc_id))
        .await
        .map_err(|_| ApiError::internal(locale))?
        .map_err(|e| map_index_error(e, locale))?
        .is_some();

    if !exists {
        return Err(ApiError::not_found(locale));
    }

    let outcome = queue.enqueue(&user.tenant_id, doc_id).map_err(|e| {
        tracing::error!(error = %e, "summary queue enqueue failed");
        ApiError::internal(locale)
    })?;

    let (enqueued, message) = match outcome {
        EnqueueOutcome::Enqueued => (true, t(locale, Msg::SummaryQueued).to_string()),
        EnqueueOutcome::AlreadyQueued => (false, t(locale, Msg::SummaryAlreadyQueued).to_string()),
    };

    Ok(Json(SummaryEnqueueResponse { enqueued, message }))
}

const MAX_SEARCH_TAG_FILTERS: usize = 20;

pub async fn search(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let tenant_id = user.tenant_id;
    let q = query.q;
    let limit = query.limit;
    let index = state.index.clone();

    let path_filter = match query.path.as_deref() {
        None | Some("") => None,
        Some(p) => Some(normalize_dir_path(p, locale)?),
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
        return Err(ApiError::bad_request(t(locale, Msg::SearchTooManyTags)));
    }
    for t in &tag_filters {
        validate_tag_format(t, locale)?;
    }

    let q_trimmed = q.trim().to_string();
    if q_trimmed.is_empty() && path_filter.is_none() && tag_filters.is_empty() {
        return Err(ApiError::bad_request(t(locale, Msg::SearchNeedsCriteria)));
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
    .map_err(|_| ApiError::internal(locale))?
    .map_err(|e| map_search_error(e, locale))?;

    Ok(Json(SearchResponse {
        total_hits,
        results: items.into_iter().map(into_search_result).collect(),
    }))
}

pub async fn catalog(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<CatalogResponse>, ApiError> {
    let dir_path = normalize_dir_path(&query.path, locale)?;
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
        .map_err(|_| ApiError::internal(locale))?
        .map_err(|e| map_index_error(e, locale))?;

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
    Extension(locale): Extension<Locale>,
    Path(doc_id): Path<u64>,
) -> Result<Json<DocumentDetailResponse>, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(t(locale, Msg::DocIdPositive)));
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
        .map_err(|_| ApiError::internal(locale))?
        .map_err(|e| map_index_error(e, locale))?;

    let d = doc.ok_or_else(|| ApiError::not_found(locale))?;
    Ok(Json(into_document_detail(d)))
}

pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Extension(locale): Extension<Locale>,
    Path(doc_id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    if doc_id == 0 {
        return Err(ApiError::bad_request(t(locale, Msg::DocIdPositive)));
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
        .map_err(|_| ApiError::internal(locale))?
        .map_err(|e| map_index_error(e, locale))?;

    if !removed {
        return Err(ApiError::not_found(locale));
    }

    Ok(StatusCode::NO_CONTENT)
}
