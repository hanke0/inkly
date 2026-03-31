mod auth;
mod cli;
mod config;
mod error;
mod routes;
mod state;
mod static_assets;

use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use clap::Parser;
use inkly_search::IndexManager;
use inkly_summarize::{Summarizer, SummarizerConfig};
use routes::{
    catalog, delete_document, get_document, healthz, index_document, index_document_upload,
    index_documents_bulk, search, session,
};
use state::AppState;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tracing::info;

use crate::cli::{Cli, Commands};

fn build_cors_layer(config: &config::Config) -> Result<CorsLayer, String> {
    if config.cors_permissive {
        return Ok(CorsLayer::permissive());
    }

    let mut origins = Vec::with_capacity(config.cors_origins.len());
    for o in &config.cors_origins {
        origins.push(HeaderValue::from_str(o).map_err(|_| format!("invalid CORS origin: {o}"))?);
    }

    if origins.is_empty() {
        return Ok(CorsLayer::permissive());
    }

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        // Preflight for multipart + Basic auth may request arbitrary client headers.
        .allow_headers(Any))
}

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Serve) {
        Commands::Serve => {
            tokio::runtime::Runtime::new()
                .expect("tokio runtime")
                .block_on(run_server());
        }
        Commands::SummaryBench {
            file,
            model,
            max_article_chars,
            runs,
            cpu,
            hf_cache,
        } => {
            dotenvy::dotenv().unwrap();
            tracing_subscriber::fmt().init();
            if let Err(e) = cli::run_summary_bench(file, model, max_article_chars, runs, cpu, hf_cache) {
                eprintln!("summary-bench: {e}");
                std::process::exit(1);
            }
        }
    }
}

async fn run_server() {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt().init();

    let config = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("startup error: {e}");
            return;
        }
    };

    let index = match IndexManager::open_or_create(&config.data_dir) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("startup error: failed to open index: {e}");
            return;
        }
    };

    let summarizer = if config.summarize_enabled {
        let hf_hub_cache = config.data_dir.join("huggingface").join("hub");
        if let Err(e) = std::fs::create_dir_all(&hf_hub_cache) {
            tracing::error!(
                path = %hf_hub_cache.display(),
                "startup error: failed to create Hugging Face cache dir: {e}"
            );
            return;
        }
        let summarizer_cfg = SummarizerConfig {
            hf_hub_cache_dir: Some(hf_hub_cache),
            ..SummarizerConfig::with_model_size(config.summarize_model)
        };
        match Summarizer::load(summarizer_cfg) {
            Ok(s) => {
                info!(model = %config.summarize_model, "summarization enabled");
                Some(s)
            }
            Err(e) => {
                tracing::error!("startup error: failed to initialize summarizer: {e}");
                return;
            }
        }
    } else {
        info!("summarization disabled; set SUMMARIZE_ENABLED=true to enable");
        None
    };

    let cors_layer = match build_cors_layer(&config) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("startup error: {e}");
            return;
        }
    };

    let state = AppState::new(index, summarizer, config.username, config.password);
    let auth_layer = axum::middleware::from_fn_with_state(state.clone(), auth::auth_middleware);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/v1/documents/upload",
            post(index_document_upload).layer(auth_layer.clone()),
        )
        .route(
            "/v1/documents/bulk",
            post(index_documents_bulk).layer(auth_layer.clone()),
        )
        .route(
            "/v1/documents/{doc_id}",
            get(get_document)
                .delete(delete_document)
                .layer(auth_layer.clone()),
        )
        .route(
            "/v1/documents",
            post(index_document).layer(auth_layer.clone()),
        )
        .route("/v1/catalog", get(catalog).layer(auth_layer.clone()))
        .route("/v1/search", get(search).layer(auth_layer.clone()))
        .route("/v1/session", get(session).layer(auth_layer))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .layer(cors_layer)
        .fallback(static_assets::spa_fallback)
        .with_state(state);

    let addr = config.host;
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("startup error: failed to bind {addr}: {e}");
            return;
        }
    };
    info!("server running on {addr}");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("server error: {e}");
    }
}
