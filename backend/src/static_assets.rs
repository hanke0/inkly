use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::response::Response;
use include_dir::{Dir, include_dir};

static DIST: Dir = include_dir!("$FRONTEND_DIST_DIR");

fn mime_for_path(path: &str) -> HeaderValue {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    HeaderValue::from_str(mime.as_ref())
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "not found").into_response()
}

pub async fn spa_fallback(OriginalUri(uri): OriginalUri) -> Response {
    let path = uri.path();
    // Disallow API fallthrough to avoid hiding errors behind `index.html`.
    if path.starts_with("/v1/") {
        return not_found();
    }

    let rel_path = path.strip_prefix('/').unwrap_or(path);
    let rel_path = if rel_path.is_empty() {
        "index.html"
    } else {
        rel_path
    };

    if let Some(file) = DIST.get_file(rel_path) {
        let ct = mime_for_path(rel_path);
        let body = Body::from(file.contents());
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, ct)
            .body(body)
            .unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "response build failed").into_response()
            });
    }

    // SPA fallback: for unknown client-side routes, serve the entrypoint.
    if let Some(index) = DIST.get_file("index.html") {
        let ct = mime_for_path("index.html");
        let body = Body::from(index.contents());
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, ct)
            .body(body)
            .unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "response build failed").into_response()
            });
    }

    (StatusCode::NOT_FOUND, "index.html missing").into_response()
}
