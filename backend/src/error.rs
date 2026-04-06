use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::i18n::{Msg, search_query_parse_detail, t};
use crate::locale::Locale;

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
    NotFound(String),
    Internal(String),
}

impl ApiError {
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn not_found(locale: Locale) -> Self {
        Self::NotFound(t(locale, Msg::NotFoundResource).to_string())
    }

    pub fn internal(locale: Locale) -> Self {
        Self::Internal(t(locale, Msg::InternalServer).to_string())
    }
}

/// Maps search-layer errors for `/v1/search` (query parse messages are user-facing).
pub fn map_search_error(e: SearchError, locale: Locale) -> ApiError {
    match e {
        SearchError::InvalidInput(msg) => {
            tracing::warn!(%msg, "invalid search query input");
            ApiError::bad_request(search_query_parse_detail(locale, &msg))
        }
        SearchError::StorageVersionMismatch { expected, found } => {
            tracing::error!(expected, found, "storage data_version mismatch");
            ApiError::internal(locale)
        }
        SearchError::LockPoisoned => {
            tracing::error!("index layer mutex poisoned");
            ApiError::internal(locale)
        }
        other => {
            tracing::error!(error = %other, "search operation failed");
            ApiError::internal(locale)
        }
    }
}

/// Maps search-layer errors for indexing and storage paths (hide raw internal strings).
pub fn map_index_error(e: SearchError, locale: Locale) -> ApiError {
    match e {
        SearchError::InvalidInput(msg) => {
            tracing::warn!(%msg, "invalid index input");
            ApiError::bad_request(t(locale, Msg::InvalidRequestGeneric).to_string())
        }
        SearchError::StorageVersionMismatch { expected, found } => {
            tracing::error!(expected, found, "storage data_version mismatch");
            ApiError::internal(locale)
        }
        SearchError::LockPoisoned => {
            tracing::error!("index layer mutex poisoned");
            ApiError::internal(locale)
        }
        other => {
            tracing::error!(error = %other, "index operation failed");
            ApiError::internal(locale)
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        let body = Json(ErrorResponse {
            code,
            error: message,
        });
        (status, body).into_response()
    }
}
