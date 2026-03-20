mod auth;
mod config;
mod error;
mod routes;
mod static_assets;
mod state;

use axum::routing::{get, post};
use axum::Router;
use inkly_search::IndexManager;
use routes::{healthz, index_document, index_documents_bulk, search};
use state::AppState;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("startup error: {e}");
            return;
        }
    };

    tracing_subscriber::fmt().init();

    let index = match IndexManager::open_or_create(&config.tantivy_dir) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("startup error: failed to open index: {e}");
            return;
        }
    };

    let state = AppState::new(index, config.jwt_secret);
    let auth_layer = axum::middleware::from_fn_with_state(state.clone(), auth::auth_middleware);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/v1/documents",
            post(index_document).layer(auth_layer.clone()),
        )
        .route(
            "/v1/documents/bulk",
            post(index_documents_bulk).layer(auth_layer.clone()),
        )
        .route("/v1/search", get(search).layer(auth_layer))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .fallback(static_assets::spa_fallback)
        .with_state(state);

    let addr = config.bind_addr;
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("startup error: failed to bind {addr}: {e}");
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        // Startup path: ok to log, but never panic.
        eprintln!("server error: {e}");
    }
}
