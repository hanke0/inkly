use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use inkly_search::SearchError;

/// JSON error body for all API failures. `error` is safe to show to users; `code` is stable for clients.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub error: String,
}

#[derive(Debug)]
pub enum ApiError {
    Unauthorized(String),
    BadRequest(String),
    NotFound,
    Internal,
}

impl ApiError {
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
}

impl From<SearchError> for ApiError {
    fn from(value: SearchError) -> Self {
        match value {
            SearchError::InvalidInput(msg) => {
                tracing::warn!(%msg, "invalid search/index input");
                ApiError::BadRequest(msg)
            }
            SearchError::StorageVersionMismatch { expected, found } => {
                tracing::error!(expected, found, "storage data_version mismatch");
                ApiError::Internal
            }
            SearchError::LockPoisoned => {
                tracing::error!("index layer mutex poisoned");
                ApiError::Internal
            }
            other => {
                tracing::error!(error = %other, "search/index operation failed");
                ApiError::Internal
            }
        }
    }
}

/// User-facing text when Tantivy/query parsing fails on `/v1/search` only.
pub(crate) fn user_message_for_search_query_error(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return "Search query could not be parsed. Simplify the query and try again.".to_string();
    }
    format!("Search query could not be parsed ({t}). Adjust the syntax and try again.")
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "The requested resource was not found. Check the document ID or path and try again."
                    .to_string(),
            ),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Something went wrong on the server. Please try again in a moment; if the problem continues, contact support."
                    .to_string(),
            ),
        };

        let body = Json(ErrorResponse { code, error: message });
        (status, body).into_response()
    }
}
