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
        tracing::error!(error = %value, "search/index operation failed");
        // Do not leak internal indexing/search details to clients.
        ApiError::Internal
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

