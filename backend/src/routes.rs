use axum::extract::{Json, Multipart, Query, State};
use axum::Extension;
use axum::response::IntoResponse;
// (no router helpers here; handlers are wired in `main.rs`)
use serde::Serialize;
use std::sync::Arc;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;

use inkly_contract::dto::{
    BulkIndexIn, DocumentIn, IndexResponse, SearchQuery, SearchResponse, SearchResult, SessionResponse,
};
use std::result::Result;

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

const MAX_TITLE: usize = 200;
const MAX_CONTENT: usize = 50_000;
const MAX_DOC_URL: usize = 2048;
const MAX_PATH: usize = 1024;
const MAX_NOTE: usize = 20_000;
const MAX_TAGS: usize = 50;
const MAX_TAG_LEN: usize = 64;

fn validate_document(input: &DocumentIn) -> Result<(), ApiError> {
    if input.doc_id == 0 {
        return Err(ApiError::bad_request("invalid doc_id"));
    }
    if input.title.len() > MAX_TITLE {
        return Err(ApiError::bad_request("invalid title"));
    }
    if input.content.len() > MAX_CONTENT {
        return Err(ApiError::bad_request("invalid content"));
    }
    if input.doc_url.len() > MAX_DOC_URL {
        return Err(ApiError::bad_request("invalid doc_url"));
    }
    if input.path.len() > MAX_PATH {
        return Err(ApiError::bad_request("invalid path"));
    }
    if input.note.len() > MAX_NOTE {
        return Err(ApiError::bad_request("invalid note"));
    }
    if input.tags.len() > MAX_TAGS {
        return Err(ApiError::bad_request("invalid tags"));
    }
    for t in &input.tags {
        if t.trim().is_empty() || t.len() > MAX_TAG_LEN {
            return Err(ApiError::bad_request("invalid tags"));
        }
    }
    Ok(())
}

pub async fn index_document(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<DocumentIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    validate_document(&input)?;

    let tenant_id = user.tenant_id;
    let doc_id = input.doc_id;
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
        doc_id = %doc_id,
        "index_document"
    );

    let stats = tokio::task::spawn_blocking(move || {
        index.index_document(
            &tenant_id,
            doc_id,
            &title,
            &content,
            &doc_url,
            &tags,
            &path,
            &note,
        )
    })
    .await
    .map_err(|_| ApiError::Internal)??;

    Ok(Json(IndexResponse {
        indexed: stats.indexed,
        deleted: stats.deleted,
    }))
}

/// Index a document whose `content` is supplied as a UTF-8 file (`multipart/form-data`, field `file`).
///
/// Other fields match `DocumentIn` as text parts: `doc_id`, `title`, `doc_url`, `path`, `note`, `tags` (comma-separated).
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
                if bytes.len() > MAX_CONTENT {
                    return Err(ApiError::bad_request("invalid content"));
                }
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

    let doc_id = doc_id_raw
        .as_deref()
        .ok_or_else(|| ApiError::bad_request("missing doc_id"))?
        .trim()
        .parse::<u64>()
        .map_err(|_| ApiError::bad_request("invalid doc_id"))?;

    let tags: Vec<String> = tags_raw
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    let input = DocumentIn {
        doc_id,
        title: title.trim().to_string(),
        content,
        doc_url: doc_url.trim().to_string(),
        tags,
        path: path.trim().to_string(),
        note,
    };

    validate_document(&input)?;

    let tenant_id = user.tenant_id;
    let doc_id = input.doc_id;
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
        doc_id = %doc_id,
        "index_document_upload"
    );

    let stats = tokio::task::spawn_blocking(move || {
        index.index_document(
            &tenant_id,
            doc_id,
            &title,
            &content,
            &doc_url,
            &tags,
            &path,
            &note,
        )
    })
    .await
    .map_err(|_| ApiError::Internal)??;

    Ok(Json(IndexResponse {
        indexed: stats.indexed,
        deleted: stats.deleted,
    }))
}

pub async fn index_documents_bulk(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<BulkIndexIn>,
) -> Result<Json<IndexResponse>, ApiError> {
    const MAX_BATCH: usize = 200;
    if input.documents.is_empty() || input.documents.len() > MAX_BATCH {
        return Err(ApiError::bad_request("invalid documents batch size"));
    }
    for doc in &input.documents {
        validate_document(doc)?;
    }

    let tenant_id = user.tenant_id;
    let docs: Vec<(u64, String, String, String, Vec<String>, String, String)> = input
        .documents
        .into_iter()
        .map(|d| {
            (
                d.doc_id,
                d.title,
                d.content,
                d.doc_url,
                d.tags,
                d.path,
                d.note,
            )
        })
        .collect();
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        indexed_documents = docs.len(),
        "index_documents_bulk"
    );

    let stats = tokio::task::spawn_blocking(move || index.index_documents(&tenant_id, docs))
        .await
        .map_err(|_| ApiError::Internal)??;

    Ok(Json(IndexResponse {
        indexed: stats.indexed,
        deleted: stats.deleted,
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

