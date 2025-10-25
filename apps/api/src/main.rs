mod error;
mod middleware;
mod models;
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
use serde::Serialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use std::path::Path;

/// SPA fallback handler - serves index.html for all non-API, non-asset routes
async fn spa_fallback_handler(uri: Uri) -> Response {
    let static_files_path = std::env::var("STATIC_FILES_PATH")
        .unwrap_or_else(|_| "../web/dist".to_string());

    let path = uri.path();

    // Try to serve the file if it exists
    let file_path = format!("{}{}", static_files_path, path);

    match tokio::fs::read_to_string(&file_path).await {
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
            } else if path.ends_with(".png") {
                return (StatusCode::OK, tokio::fs::read(&file_path).await.unwrap()).into_response();
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                return (StatusCode::OK, tokio::fs::read(&file_path).await.unwrap()).into_response();
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "text/plain"
            };

            (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, content_type)], contents).into_response()
        }
        Err(_) => {
            // File doesn't exist, serve index.html for SPA routing
            let index_path = format!("{}/index.html", static_files_path);
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
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    tracing::info!("✅ Database connected successfully");

    // Run migrations (commented out to avoid prepared statement conflicts with pooler)
    // Note: Migrations have already been applied to the database
    // tracing::info!("🔄 Running database migrations...");
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");
    // tracing::info!("✅ Migrations completed successfully");

    // Create app state
    let state = AppState { db: pool };

    // Build protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/auth/me", get(routes::auth::me))
        .layer(axum_middleware::from_fn(auth_middleware));

    // Build admin-only routes (require ADMIN role)
    let admin_routes = Router::new()
        .route("/api/questions", post(routes::questions::create_question))
        .route("/api/questions/:id", put(routes::questions::update_question))
        .route("/api/questions/:id", delete(routes::questions::delete_question))
        .route("/api/questions/:question_id/answers", post(routes::answers::create_answer))
        .route("/api/answers/:id", put(routes::answers::update_answer))
        .route("/api/answers/:id", delete(routes::answers::delete_answer))
        // Admin dashboard routes
        .route("/api/admin/sessions", get(routes::admin::list_sessions))
        .route("/api/admin/stats", get(routes::admin::get_stats))
        .route("/api/admin/audit-logs", get(routes::admin::get_audit_logs))
        // Issues management routes
        .route("/api/admin/issues", get(routes::issues::list_issues))
        .route("/api/admin/issues", post(routes::issues::create_issue))
        .route("/api/admin/issues/:category/tree", get(routes::issues::get_issue_tree))
        .route("/api/admin/issues/:category/graph", get(routes::issues::get_issue_graph))
        .route("/api/admin/issues/:category", put(routes::issues::update_issue))
        .route("/api/admin/issues/:category", delete(routes::issues::delete_issue))
        .route("/api/admin/issues/:category/toggle", patch(routes::issues::toggle_issue))
        // Node routes (NODE-GRAPH)
        .route("/api/nodes", get(routes::nodes::list_nodes))
        .route("/api/nodes/:id", get(routes::nodes::get_node))
        .route("/api/nodes/:id/with-connections", get(routes::nodes::get_node_with_connections))
        .route("/api/nodes", post(routes::nodes::create_node))
        .route("/api/nodes/:id", put(routes::nodes::update_node))
        .route("/api/nodes/:id", delete(routes::nodes::delete_node))
        // Connection routes (NODE-GRAPH)
        .route("/api/connections", get(routes::connections::list_connections))
        .route("/api/connections", post(routes::connections::create_connection))
        .route("/api/connections/:id", put(routes::connections::update_connection))
        .route("/api/connections/:id", delete(routes::connections::delete_connection))
        .layer(axum_middleware::from_fn(middleware::auth::require_admin));

    // Get static files path from environment or use default
    let static_files_path = std::env::var("STATIC_FILES_PATH")
        .unwrap_or_else(|_| "../web/dist".to_string());

    tracing::info!("📁 Static files path: {}", static_files_path);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(health_check_db))
        // Authentication routes (public)
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/refresh", post(routes::auth::refresh))
        // Questions routes (public read)
        .route("/api/questions", get(routes::questions::list_questions))
        .route("/api/questions/:id", get(routes::questions::get_question))
        // Answers routes (public read)
        .route("/api/questions/:question_id/answers", get(routes::answers::list_answers))
        // Troubleshooting routes (public)
        .route("/api/troubleshoot/start", post(routes::troubleshoot::start_session))
        .route("/api/troubleshoot/:session_id", get(routes::troubleshoot::get_session))
        .route("/api/troubleshoot/:session_id/answer", post(routes::troubleshoot::submit_answer))
        .route("/api/troubleshoot/:session_id/history", get(routes::troubleshoot::get_session_history))
        // Merge protected routes
        .merge(protected_routes)
        // Merge admin routes
        .merge(admin_routes)
        // Demo error endpoints
        .route("/api/demo/not-found", get(demo_not_found))
        .route("/api/demo/unauthorized", get(demo_unauthorized))
        .route("/api/demo/validation", get(demo_validation))
        .layer(CorsLayer::permissive())
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
        .expect(&format!("Invalid HOST:PORT combination: {}", addr_str));

    tracing::info!("🚀 Equipment Troubleshooting System");

    // Check if HTTPS is requested via environment variables
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_default();
    let use_https = frontend_url.starts_with("https://");

    // Check for SSL certificate files in the project root
    let cert_path = Path::new("../../server.crt");
    let key_path = Path::new("../../server.key");

    if use_https {
        // HTTPS mode requested via .env
        if !cert_path.exists() || !key_path.exists() {
            tracing::error!("❌ HTTPS requested (FRONTEND_URL starts with https://) but SSL certificates not found!");
            tracing::error!("📝 Please add server.crt and server.key to the project root");
            tracing::error!("📖 See SSL_SETUP.md for instructions");
            panic!("SSL certificates required but not found");
        }

        tracing::info!("🔒 HTTPS enabled (detected from FRONTEND_URL in .env)");
        tracing::info!("📡 Server listening on https://{}", addr);
        tracing::info!("🌐 Frontend & API available at https://{}", addr);

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
