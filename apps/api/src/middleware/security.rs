use axum::{
    extract::Request,
    http::{header, HeaderName},
    middleware::Next,
    response::Response,
};

/// Middleware to add security headers to all responses
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Strict-Transport-Security: Force HTTPS for 1 year
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    // X-Frame-Options: Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        "DENY".parse().unwrap(),
    );

    // X-Content-Type-Options: Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // X-XSS-Protection: Enable browser XSS protection
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );

    // Referrer-Policy: Control referrer information
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Content-Security-Policy: Restrict resource loading
    // Allow same-origin and inline styles/scripts (needed for React/Vite)
    let csp = "default-src 'self'; \
               script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
               style-src 'self' 'unsafe-inline'; \
               img-src 'self' data:; \
               font-src 'self' data:; \
               connect-src 'self'; \
               frame-ancestors 'none'";
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        csp.parse().unwrap(),
    );

    // Permissions-Policy: Disable unnecessary browser features
    let permissions = "geolocation=(), microphone=(), camera=(), payment=()";
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        permissions.parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert!(headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert!(headers.contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
        assert!(headers.contains_key(header::REFERRER_POLICY));

        // Verify header values
        assert_eq!(
            headers.get(header::X_FRAME_OPTIONS).unwrap(),
            "DENY"
        );
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );
    }
}
