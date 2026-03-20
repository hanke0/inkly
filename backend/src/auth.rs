use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    tenant_id: String,
    _exp: usize,
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

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("invalid Authorization header format"))?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let decoded = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| ApiError::unauthorized("invalid token"))?;

    let user = AuthUser {
        user_id: decoded.claims.sub,
        tenant_id: decoded.claims.tenant_id,
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

