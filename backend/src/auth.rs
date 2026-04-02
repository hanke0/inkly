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
        .ok_or_else(|| {
            ApiError::unauthorized(
                "Sign-in required. Send an `Authorization: Basic …` header or open the app and sign in again.",
            )
        })?;

    let b64 = auth_header
        .strip_prefix("Basic ")
        .ok_or_else(|| {
            ApiError::unauthorized(
                "Authorization must use Basic authentication (`Authorization: Basic <base64>`). Check the header format.",
            )
        })?
        .trim();

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| {
            ApiError::unauthorized(
                "Credentials could not be decoded. Re-enter your username and password and try again.",
            )
        })?;

    let creds = String::from_utf8(decoded).map_err(|_| {
        ApiError::unauthorized(
            "Credentials could not be decoded. Re-enter your username and password and try again.",
        )
    })?;

    let (username, password) = creds
        .split_once(':')
        .unwrap_or((creds.as_str(), ""));

    if !basic_credentials_match(
        username,
        password,
        state.expected_username(),
        state.expected_password(),
    ) {
        return Err(ApiError::unauthorized(
            "Username or password did not match the server configuration. Check your credentials and try again.",
        ));
    }

    let user = AuthUser {
        user_id: username.to_string(),
        tenant_id: username.to_string(),
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

fn basic_credentials_match(
    provided_user: &str,
    provided_pass: &str,
    expected_user: &str,
    expected_pass: &str,
) -> bool {
    use subtle::ConstantTimeEq;

    // ct_eq requires equal-length slices.  Pad both sides to the longer length
    // so neither the content comparison nor the length check leaks timing info.
    fn ct_str_eq(a: &[u8], b: &[u8]) -> subtle::Choice {
        const BUF: usize = 256;
        let mut ab = [0u8; BUF];
        let mut bb = [0u8; BUF];
        let al = a.len().min(BUF);
        let bl = b.len().min(BUF);
        ab[..al].copy_from_slice(&a[..al]);
        bb[..bl].copy_from_slice(&b[..bl]);
        (a.len() as u64).ct_eq(&(b.len() as u64)) & ab.ct_eq(&bb)
    }

    bool::from(
        ct_str_eq(provided_user.as_bytes(), expected_user.as_bytes())
            & ct_str_eq(provided_pass.as_bytes(), expected_pass.as_bytes()),
    )
}
