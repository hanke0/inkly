use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use inkly_search::SearchError;

#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
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
                ApiError::BadRequest(format!(
                    "storage data_version mismatch: expected {expected}, found {found}"
                ))
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

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

