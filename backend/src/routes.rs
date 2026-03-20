use axum::extract::{Json, Query, State};
use axum::Extension;
use axum::response::IntoResponse;
// (no router helpers here; handlers are wired in `main.rs`)
use serde::Serialize;
use std::sync::Arc;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;

use inkly_contract::dto::{BulkIndexIn, DocumentIn, IndexResponse, SearchQuery, SearchResponse, SearchResult};
use std::result::Result;

use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn healthz() -> impl IntoResponse {
    axum::Json(HealthResponse { status: "ok" })
}

fn validate_document(input: &DocumentIn) -> Result<(), ApiError> {
    const MAX_DOC_ID: usize = 128;
    const MAX_TITLE: usize = 200;
    const MAX_CONTENT: usize = 50_000;

    if input.doc_id.trim().is_empty() || input.doc_id.len() > MAX_DOC_ID {
        return Err(ApiError::bad_request("invalid doc_id"));
    }
    if input.title.len() > MAX_TITLE {
        return Err(ApiError::bad_request("invalid title"));
    }
    if input.content.len() > MAX_CONTENT {
        return Err(ApiError::bad_request("invalid content"));
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
    let index = state.index.clone();

    info!(
        tenant_id = %tenant_id,
        user_id = %user.user_id,
        doc_id = %doc_id,
        "index_document"
    );

    let stats = tokio::task::spawn_blocking(move || index.index_document(&tenant_id, &doc_id, &title, &content))
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
    let docs: Vec<(String, String, String)> = input
        .documents
        .into_iter()
        .map(|d| (d.doc_id, d.title, d.content))
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
            snippet: it.snippet,
            score: it.score,
        })
        .collect();

    Ok(Json(SearchResponse {
        total_hits,
        results,
    }))
}

