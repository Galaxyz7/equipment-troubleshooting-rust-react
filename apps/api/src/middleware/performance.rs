use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Performance monitoring middleware
/// Logs request duration and adds timing header
pub async fn performance_monitoring_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log slow requests (>500ms)
    if duration.as_millis() > 500 {
        tracing::warn!(
            "⚠️  SLOW REQUEST: {} {} - {}ms (status: {})",
            method,
            uri,
            duration.as_millis(),
            status
        );
    } else {
        tracing::debug!(
            "{} {} - {}ms (status: {})",
            method,
            uri,
            duration.as_millis(),
            status
        );
    }

    response
}
