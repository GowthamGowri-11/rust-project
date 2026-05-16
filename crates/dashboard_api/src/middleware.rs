use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::info;

pub async fn log_request(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    info!(
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

pub async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
