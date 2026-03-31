use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
    pub tenant_id: String,
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing Authorization header"))?;

    let b64 = auth_header
        .strip_prefix("Basic ")
        .ok_or_else(|| ApiError::unauthorized("invalid Authorization header format"))?
        .trim();

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| ApiError::unauthorized("invalid credentials"))?;

    let creds = String::from_utf8(decoded).map_err(|_| ApiError::unauthorized("invalid credentials"))?;

    let (username, password) = creds
        .split_once(':')
        .unwrap_or((creds.as_str(), ""));

    if !basic_credentials_match(
        username,
        password,
        state.expected_username(),
        state.expected_password(),
    ) {
        return Err(ApiError::unauthorized("invalid credentials"));
    }

    let user = AuthUser {
        user_id: username.to_string(),
        tenant_id: username.to_string(),
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

fn basic_credentials_match(provided_user: &str, provided_pass: &str, expected_user: &str, expected_pass: &str) -> bool {
    use subtle::ConstantTimeEq;

    if provided_user.len() != expected_user.len() || provided_pass.len() != expected_pass.len() {
        return false;
    }

    let user_ok = provided_user.as_bytes().ct_eq(expected_user.as_bytes());
    let pass_ok = provided_pass.as_bytes().ct_eq(expected_pass.as_bytes());
    bool::from(user_ok & pass_ok)
}
