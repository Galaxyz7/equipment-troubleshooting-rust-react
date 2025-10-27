mod error;
mod middleware;
mod models;
mod openapi;
mod routes;
mod utils;

use axum::{
    extract::State,
    http::{StatusCode, Uri},
    middleware as axum_middleware,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use error::{ApiError, ApiResult};
use equipment_troubleshooting::AppState;
use middleware::auth::auth_middleware;
use middleware::performance::performance_monitoring_middleware;
use middleware::rate_limit::{rate_limit_middleware, RateLimiter, RateLimiterExtension};
use middleware::security::security_headers_middleware;
use openapi::ApiDoc;
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use axum::http::{Method, header};
use std::path::{Path, PathBuf};
use std::fs;

/// SPA fallback handler - serves index.html for all non-API, non-asset routes
async fn spa_fallback_handler(uri: Uri) -> Response {
    let static_files_path = std::env::var("STATIC_FILES_PATH")
        .unwrap_or_else(|_| "../web/dist".to_string());

    let path = uri.path();

    // SECURITY: Prevent path traversal attacks
    // Canonicalize base path to get absolute path
    let base_path = match fs::canonicalize(&static_files_path) {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!("Static files path does not exist: {}", static_files_path);
            return (StatusCode::NOT_FOUND, "Frontend not built").into_response();
        }
    };

    // Build requested file path - remove leading slash to avoid absolute path interpretation
    let requested_file = path.trim_start_matches('/');
    let file_path = base_path.join(requested_file);

    // Canonicalize the requested path and verify it's within base_path
    // If the file doesn't exist yet, check if parent directory is within base_path
    let safe_path = match fs::canonicalize(&file_path) {
        Ok(canonical) => {
            // File exists - verify it's within base directory
            if !canonical.starts_with(&base_path) {
                tracing::warn!("Path traversal attempt blocked: {:?}", path);
                return (StatusCode::FORBIDDEN, "Access denied").into_response();
            }
            canonical
        }
        Err(_) => {
            // File doesn't exist - verify parent directory is within base_path
            if let Some(parent) = file_path.parent() {
                if let Ok(canonical_parent) = fs::canonicalize(parent) {
                    if !canonical_parent.starts_with(&base_path) {
                        tracing::warn!("Path traversal attempt blocked: {:?}", path);
                        return (StatusCode::FORBIDDEN, "Access denied").into_response();
                    }
                }
                // Parent doesn't exist, will fall through to index.html
            }
            file_path.clone()
        }
    };

    match tokio::fs::read_to_string(&safe_path).await {
        Ok(contents) => {
            // Determine content type based on file extension
            let content_type = if path.ends_with(".html") {
                "text/html"
            } else if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".json") {
                "application/json"
            } else if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") {
                return (StatusCode::OK, tokio::fs::read(&safe_path).await.unwrap()).into_response();
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "text/plain"
            };

            (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, content_type)], contents).into_response()
        }
        Err(_) => {
            // File doesn't exist, serve index.html for SPA routing
            let index_path = base_path.join("index.html");
            match tokio::fs::read_to_string(&index_path).await {
                Ok(contents) => Html(contents).into_response(),
                Err(_) => (StatusCode::NOT_FOUND, "Frontend not built").into_response(),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get frontend URL for CORS configuration
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| {
            tracing::warn!("⚠️  FRONTEND_URL not set, defaulting to http://localhost:5173");
            "http://localhost:5173".to_string()
        });

    // Validate JWT_SECRET is set (critical security requirement)
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("❌ CRITICAL: JWT_SECRET must be set in .env file for authentication to work");

    if jwt_secret.len() < 32 {
        panic!("❌ CRITICAL: JWT_SECRET must be at least 32 characters long for security");
    }

    tracing::info!("✅ JWT_SECRET validated ({} characters)", jwt_secret.len());

    // Get database URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Create database connection pool with disabled statement caching
    // Note: Supabase pooler requires statement_cache_capacity=0 to avoid conflicts
    tracing::info!("📦 Connecting to database...");
    let connect_options = PgConnectOptions::from_str(&database_url)
        .expect("Invalid DATABASE_URL")
        .statement_cache_capacity(0); // Disable prepared statements for Supabase pooler

    let pool = PgPoolOptions::new()
        .max_connections(20) // Increased from 5 to 20 for better concurrency
        .min_connections(2)  // Maintain 2 connections ready
        .acquire_timeout(std::time::Duration::from_secs(3)) // 3s timeout
        .idle_timeout(Some(std::time::Duration::from_secs(600))) // 10 min idle timeout
        .connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    tracing::info!("✅ Database connected successfully (pool: 2-20 connections)");

    // Run migrations (commented out to avoid prepared statement conflicts with pooler)
    // Note: Migrations have already been applied to the database
    // tracing::info!("🔄 Running database migrations...");
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");
    // tracing::info!("✅ Migrations completed successfully");

    // Create app state with caching layer
    let state = AppState::new(pool);
    tracing::info!("💾 Performance caching enabled (questions: 5min, trees/graphs: 10min)");

    // Create rate limiter (100 requests per 60 seconds per IP)
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));
    tracing::info!("🚦 Rate limiter initialized (100 requests/60 seconds)");

    // Spawn background task to clean up old rate limit entries every 5 minutes
    // This prevents memory leak by removing expired entries from the HashMap
    {
        let rate_limiter_cleanup = Arc::clone(&rate_limiter);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                rate_limiter_cleanup.cleanup().await;
                tracing::debug!("🧹 Rate limiter cleanup completed");
            }
        });
        tracing::info!("🧹 Rate limiter cleanup task started (runs every 5 minutes)");
    }

    // Build protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/auth/me", get(routes::auth::me))
        .layer(axum_middleware::from_fn(auth_middleware));

    // Build admin-only routes (require ADMIN role)
    let admin_routes = Router::new()
        // Admin dashboard routes
        .route("/api/v1/admin/sessions", get(routes::admin::list_sessions))
        .route("/api/v1/admin/sessions", delete(routes::admin::delete_sessions))
        .route("/api/v1/admin/sessions/count", get(routes::admin::count_sessions))
        .route("/api/v1/admin/stats", get(routes::admin::get_stats))
        .route("/api/v1/admin/audit-logs", get(routes::admin::get_audit_logs))
        .route("/api/v1/admin/performance", get(routes::admin::get_performance_metrics))
        // Category management routes
        .route("/api/v1/admin/categories", get(routes::admin::list_categories))
        .route("/api/v1/admin/categories/:name", put(routes::admin::rename_category).delete(routes::admin::delete_category))
        // Issues management routes
        .route("/api/v1/admin/issues", get(routes::issues::list_issues))
        .route("/api/v1/admin/issues", post(routes::issues::create_issue))
        // Import/Export routes (must come before /:category routes to avoid conflicts)
        .route("/api/v1/admin/issues/export-all", get(routes::issues::export_all_issues))
        .route("/api/v1/admin/issues/import", post(routes::issues::import_issues))
        .route("/api/v1/admin/issues/:category/graph", get(routes::issues::get_issue_graph))
        .route("/api/v1/admin/issues/:category/export", get(routes::issues::export_issue))
        .route("/api/v1/admin/issues/:category", put(routes::issues::update_issue))
        .route("/api/v1/admin/issues/:category", delete(routes::issues::delete_issue))
        .route("/api/v1/admin/issues/:category/toggle", patch(routes::issues::toggle_issue))
        // Node routes (NODE-GRAPH)
        .route("/api/v1/nodes", get(routes::nodes::list_nodes))
        .route("/api/v1/nodes/:id", get(routes::nodes::get_node))
        .route("/api/v1/nodes/:id/with-connections", get(routes::nodes::get_node_with_connections))
        .route("/api/v1/nodes", post(routes::nodes::create_node))
        .route("/api/v1/nodes/:id", put(routes::nodes::update_node))
        .route("/api/v1/nodes/:id", delete(routes::nodes::delete_node))
        // Connection routes (NODE-GRAPH)
        .route("/api/v1/connections", get(routes::connections::list_connections))
        .route("/api/v1/connections", post(routes::connections::create_connection))
        .route("/api/v1/connections/:id", put(routes::connections::update_connection))
        .route("/api/v1/connections/:id", delete(routes::connections::delete_connection))
        .layer(axum_middleware::from_fn(middleware::auth::require_admin));

    // Get static files path from environment or use default
    let static_files_path = std::env::var("STATIC_FILES_PATH")
        .unwrap_or_else(|_| "../web/dist".to_string());

    tracing::info!("📁 Static files path: {}", static_files_path);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/health", get(health_check_db))
        // OpenAPI/Swagger documentation with enhanced configuration
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(utoipa_swagger_ui::Config::default()
                    .try_it_out_enabled(true)  // Enable "Try it out" by default
                    .filter(true)               // Enable search/filter
                    .persist_authorization(true) // Remember auth token
                    .display_request_duration(true) // Show request timing
                    .doc_expansion("list")      // Expand tags, not operations
                )
        )
        // Authentication routes (public)
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        // Troubleshooting routes (public)
        .route("/api/v1/troubleshoot/start", post(routes::troubleshoot::start_session))
        .route("/api/v1/troubleshoot/:session_id", get(routes::troubleshoot::get_session))
        .route("/api/v1/troubleshoot/:session_id/answer", post(routes::troubleshoot::submit_answer))
        .route("/api/v1/troubleshoot/:session_id/history", get(routes::troubleshoot::get_session_history))
        // Merge protected routes
        .merge(protected_routes)
        // Merge admin routes
        .merge(admin_routes)
        // Demo error endpoints
        .route("/api/v1/demo/not-found", get(demo_not_found))
        .route("/api/v1/demo/unauthorized", get(demo_unauthorized))
        .route("/api/v1/demo/validation", get(demo_validation))
        .layer(axum_middleware::from_fn(performance_monitoring_middleware))
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .layer(axum_middleware::from_fn(rate_limit_middleware))
        .layer(axum::Extension(RateLimiterExtension(rate_limiter)))
        // SECURITY: Configure CORS to only allow specific origins instead of permissive
        .layer(
            CorsLayer::new()
                .allow_origin(frontend_url.parse::<axum::http::HeaderValue>()
                    .unwrap_or_else(|_| {
                        tracing::error!("Invalid FRONTEND_URL: {}", frontend_url);
                        "http://localhost:5173".parse().unwrap()
                    }))
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                ])
                .allow_credentials(true)
        )
        .with_state(state)
        // Serve static files for SPA (fallback to index.html for client-side routing)
        .fallback(spa_fallback_handler);

    // Get host from env or use default
    let host = std::env::var("HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    // Get port from env or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    // Parse the host and port into a SocketAddr
    let addr_str = format!("{}:{}", host, port);
    let addr = addr_str.parse::<SocketAddr>()
        .unwrap_or_else(|_| panic!("Invalid HOST:PORT combination: {}", addr_str));

    tracing::info!("🚀 Equipment Troubleshooting System");

    // Check if HTTPS is requested via environment variables
    let use_https = frontend_url.starts_with("https://");

    // Function to find first .crt and .key files in a directory
    let find_ssl_certs = |dir: &str| -> Option<(PathBuf, PathBuf)> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return None;
        }

        let entries = fs::read_dir(dir_path).ok()?;
        let mut cert_file: Option<PathBuf> = None;
        let mut key_file: Option<PathBuf> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "crt" && cert_file.is_none() {
                    cert_file = Some(path);
                } else if ext == "key" && key_file.is_none() {
                    key_file = Some(path);
                }
            }

            // Stop if we found both
            if cert_file.is_some() && key_file.is_some() {
                break;
            }
        }

        match (cert_file, key_file) {
            (Some(cert), Some(key)) => Some((cert, key)),
            _ => None,
        }
    };

    // Try to find SSL certificates in deployment dir first, then project root
    let ssl_certs = find_ssl_certs(".")
        .or_else(|| find_ssl_certs("../.."));

    let (cert_path, key_path) = ssl_certs
        .unwrap_or_else(|| (PathBuf::from("./server.crt"), PathBuf::from("./server.key")));

    if use_https {
        // HTTPS mode requested via .env
        if !cert_path.exists() || !key_path.exists() {
            tracing::error!("❌ HTTPS requested (FRONTEND_URL starts with https://) but SSL certificates not found!");
            tracing::error!("📝 Please add any .crt and .key file to the same directory as the binary");
            tracing::error!("📖 See SSL_SETUP.md for instructions");
            panic!("SSL certificates required but not found");
        }

        tracing::info!("🔒 HTTPS enabled (detected from FRONTEND_URL in .env)");
        tracing::info!("📜 Using certificate: {}", cert_path.display());
        tracing::info!("🔑 Using key: {}", key_path.display());
        tracing::info!("📡 Server listening on https://{}", addr);
        tracing::info!("🌐 Frontend & API available at https://{}", addr);
        tracing::info!("📚 API Documentation (Swagger UI) available at https://{}/swagger-ui", addr);

        let config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            cert_path,
            key_path,
        )
        .await
        .expect("Failed to load SSL certificates");

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("Failed to start HTTPS server");
    } else {
        // HTTP mode
        tracing::info!("📡 Starting HTTP server (FRONTEND_URL in .env uses http://)");
        tracing::info!("💡 To enable HTTPS, change FRONTEND_URL to https:// and add SSL certificates");
        tracing::info!("📡 Server listening on http://{}", addr);
        tracing::info!("🌐 Frontend & API available at http://{}", addr);
        tracing::info!("📚 API Documentation (Swagger UI) available at http://{}/swagger-ui", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");

        axum::serve(listener, app)
            .await
            .expect("Failed to start server");
    }
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
}

async fn health_check_db(State(state): State<AppState>) -> Json<HealthResponse> {
    // Test database connection with a simple query
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
    })
}

// ============================================
// DEMO ERROR ENDPOINTS
// ============================================

/// Demo: Not Found error (404)
async fn demo_not_found() -> ApiResult<Json<String>> {
    Err(ApiError::not_found("The requested resource does not exist"))
}

/// Demo: Unauthorized error (401)
async fn demo_unauthorized() -> ApiResult<Json<String>> {
    Err(ApiError::unauthorized("Authentication required"))
}

/// Demo: Validation error (422)
async fn demo_validation() -> ApiResult<Json<String>> {
    Err(ApiError::validation(vec![
        ("email".to_string(), "Invalid email format".to_string()),
        ("password".to_string(), "Password must be at least 8 characters".to_string()),
    ]))
}
