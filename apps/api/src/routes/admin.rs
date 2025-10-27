use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::utils::audit;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Extension;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use ts_rs::TS;
use uuid::Uuid;

/// Session summary for admin list view
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct SessionSummary {
    pub session_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub abandoned: bool,
    pub tech_identifier: Option<String>,
    pub client_site: Option<String>,
    pub final_conclusion: Option<String>,
    pub step_count: i32,
}

/// Response for admin sessions list
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct SessionsListResponse {
    pub sessions: Vec<SessionSummary>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Dashboard statistics response
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct DashboardStats {
    #[ts(type = "number")]
    pub total_sessions: i64,
    #[ts(type = "number")]
    pub completed_sessions: i64,
    #[ts(type = "number")]
    pub abandoned_sessions: i64,
    #[ts(type = "number")]
    pub active_sessions: i64,
    pub avg_steps_to_completion: f64,
    pub most_common_conclusions: Vec<ConclusionStats>,
    pub sessions_by_category: Vec<CategoryStats>,
}

/// Statistics for a specific conclusion
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ConclusionStats {
    pub conclusion: String,
    #[ts(type = "number")]
    pub count: i64,
}

/// Statistics by category
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CategoryStats {
    pub category: String,
    #[ts(type = "number")]
    pub count: i64,
}

/// Audit log entry
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub user_id: Option<i32>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    #[ts(skip)]
    pub changes: serde_json::Value,
}

/// Response for audit logs list
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLogEntry>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Query parameters for sessions list endpoint
#[derive(Debug, Deserialize)]
pub struct SessionsQueryParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    pub category: Option<String>,
    pub status: Option<String>, // "completed", "abandoned", "active"
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub search: Option<String>, // Search in tech_identifier, client_site
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    50
}

/// Query parameters for stats endpoint
#[derive(Debug, Deserialize)]
pub struct StatsQueryParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Query parameters for delete sessions endpoint
#[derive(Debug, Deserialize)]
pub struct DeleteSessionsParams {
    pub time_range: Option<String>, // "all_time", "past_month", "past_week", "today"
    pub category: Option<String>,   // Issue category to filter by
    pub status: Option<String>,     // "all", "completed", "abandoned", "active"
}

/// Response for delete sessions endpoint
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct DeleteSessionsResponse {
    pub deleted_count: i64,
}

/// GET /api/admin/sessions
/// List all sessions with pagination and filters (ADMIN only)
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<SessionsQueryParams>,
) -> ApiResult<Json<SessionsListResponse>> {
    let page = params.page;
    let page_size = params.page_size.min(200); // Cap at 200
    let offset = (page - 1) * page_size;

    // Build query safely using QueryBuilder to prevent SQL injection
    use sqlx::QueryBuilder;

    // Build count query first
    let mut count_query = QueryBuilder::new("SELECT COUNT(*) FROM sessions WHERE 1=1");

    if let Some(status) = &params.status {
        match status.as_str() {
            "completed" => {
                count_query.push(" AND completed_at IS NOT NULL");
            }
            "abandoned" => {
                count_query.push(" AND abandoned = true");
            }
            "active" => {
                count_query.push(" AND completed_at IS NULL");
                count_query.push(" AND abandoned = false");
            }
            _ => {}
        }
    }

    if let Some(start_date) = &params.start_date {
        count_query.push(" AND started_at >= ");
        count_query.push_bind(start_date);
    }

    if let Some(end_date) = &params.end_date {
        count_query.push(" AND started_at <= ");
        count_query.push_bind(end_date);
    }

    if let Some(search) = &params.search {
        count_query.push(" AND (tech_identifier ILIKE ");
        count_query.push_bind(format!("%{}%", search));
        count_query.push(" OR client_site ILIKE ");
        count_query.push_bind(format!("%{}%", search));
        count_query.push(")");
    }

    if let Some(category) = &params.category {
        count_query.push(" AND (steps->0->>'category')::text = ");
        count_query.push_bind(category);
    }

    // Execute count query
    let total_count = match count_query.build_query_scalar::<i64>()
        .fetch_one(&state.db)
        .await {
            Ok(count) => count,
            Err(e) => {
                tracing::error!("❌ Error fetching session count: {:?}", e);
                if e.to_string().contains("relation") && e.to_string().contains("does not exist") {
                    tracing::warn!("⚠️  Sessions table does not exist. Returning empty list.");
                }
                // Return empty list instead of error
                return Ok(Json(SessionsListResponse {
                    sessions: vec![],
                    total_count: 0,
                    page,
                    page_size,
                }));
            }
        };

    // Build sessions query with same filters
    let mut sessions_query = QueryBuilder::new(
        "SELECT session_id, started_at, completed_at, abandoned, \
         tech_identifier, client_site, final_conclusion, \
         COALESCE(jsonb_array_length(steps), 0)::int as step_count \
         FROM sessions WHERE 1=1"
    );

    if let Some(status) = &params.status {
        match status.as_str() {
            "completed" => {
                sessions_query.push(" AND completed_at IS NOT NULL");
            }
            "abandoned" => {
                sessions_query.push(" AND abandoned = true");
            }
            "active" => {
                sessions_query.push(" AND completed_at IS NULL");
                sessions_query.push(" AND abandoned = false");
            }
            _ => {}
        }
    }

    if let Some(start_date) = &params.start_date {
        sessions_query.push(" AND started_at >= ");
        sessions_query.push_bind(start_date);
    }

    if let Some(end_date) = &params.end_date {
        sessions_query.push(" AND started_at <= ");
        sessions_query.push_bind(end_date);
    }

    if let Some(search) = &params.search {
        sessions_query.push(" AND (tech_identifier ILIKE ");
        sessions_query.push_bind(format!("%{}%", search));
        sessions_query.push(" OR client_site ILIKE ");
        sessions_query.push_bind(format!("%{}%", search));
        sessions_query.push(")");
    }

    if let Some(category) = &params.category {
        sessions_query.push(" AND (steps->0->>'category')::text = ");
        sessions_query.push_bind(category);
    }

    sessions_query.push(" ORDER BY started_at DESC LIMIT ");
    sessions_query.push_bind(page_size);
    sessions_query.push(" OFFSET ");
    sessions_query.push_bind(offset);

    // Execute sessions query
    let sessions = match sessions_query.build_query_as::<(
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
        bool,
        Option<String>,
        Option<String>,
        Option<String>,
        i32,
    )>()
    .fetch_all(&state.db)
    .await {
        Ok(sessions) => sessions,
        Err(e) => {
            tracing::error!("❌ Error fetching sessions: {:?}", e);
            // Return empty list instead of error
            return Ok(Json(SessionsListResponse {
                sessions: vec![],
                total_count: 0,
                page,
                page_size,
            }));
        }
    };

    let session_summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|s| SessionSummary {
            session_id: s.0,
            started_at: s.1.to_rfc3339(),
            completed_at: s.2.map(|dt| dt.to_rfc3339()),
            abandoned: s.3,
            tech_identifier: s.4,
            client_site: s.5,
            final_conclusion: s.6,
            step_count: s.7,
        })
        .collect();

    Ok(Json(SessionsListResponse {
        sessions: session_summaries,
        total_count,
        page,
        page_size,
    }))
}

/// GET /api/admin/stats
/// Get dashboard statistics (ADMIN only) - OPTIMIZED to single query with CTEs
pub async fn get_stats(
    State(state): State<AppState>,
    Query(params): Query<StatsQueryParams>,
) -> ApiResult<Json<DashboardStats>> {
    // Build query safely with optional date filters using CASE/COALESCE
    // This avoids string concatenation while maintaining the CTE structure
    let query_with_binds = sqlx::query(
        r#"
        WITH filtered_sessions AS (
            SELECT
                session_id,
                started_at,
                completed_at,
                abandoned,
                final_conclusion,
                steps
            FROM sessions
            WHERE ($1::timestamp IS NULL OR started_at >= $1::timestamp)
              AND ($2::timestamp IS NULL OR started_at <= $2::timestamp)
        ),
        basic_stats AS (
            SELECT
                COALESCE(COUNT(*), 0) as total,
                COALESCE(COUNT(*) FILTER (WHERE completed_at IS NOT NULL), 0) as completed,
                -- Abandoned = explicitly marked OR incomplete sessions older than 1 hour
                COALESCE(COUNT(*) FILTER (
                    WHERE abandoned = true
                    OR (completed_at IS NULL AND started_at <= NOW() - INTERVAL '1 hour')
                ), 0) as abandoned,
                -- Active = incomplete, not abandoned, and started within the last hour
                COALESCE(COUNT(*) FILTER (
                    WHERE completed_at IS NULL
                    AND abandoned = false
                    AND started_at > NOW() - INTERVAL '1 hour'
                ), 0) as active,
                -- Average steps only for completed sessions with valid steps data
                COALESCE(AVG(jsonb_array_length(steps)) FILTER (
                    WHERE completed_at IS NOT NULL
                    AND steps IS NOT NULL
                    AND jsonb_array_length(steps) > 0
                ), 0.0) as avg_steps
            FROM filtered_sessions
        ),
        conclusion_stats AS (
            SELECT final_conclusion, COUNT(*) as count
            FROM filtered_sessions
            WHERE final_conclusion IS NOT NULL
            GROUP BY final_conclusion
            ORDER BY count DESC
            LIMIT 10
        ),
        category_stats AS (
            SELECT
                COALESCE((steps->0->>'category')::text, 'unknown') as category,
                COUNT(*) as count
            FROM filtered_sessions
            WHERE steps IS NOT NULL AND jsonb_array_length(steps) > 0
            GROUP BY category
            ORDER BY count DESC
        )
        SELECT
            COALESCE((SELECT total FROM basic_stats), 0) as total_sessions,
            COALESCE((SELECT completed FROM basic_stats), 0) as completed_sessions,
            COALESCE((SELECT abandoned FROM basic_stats), 0) as abandoned_sessions,
            COALESCE((SELECT active FROM basic_stats), 0) as active_sessions,
            COALESCE((SELECT avg_steps FROM basic_stats), 0.0) as avg_steps_to_completion,
            COALESCE(
                (SELECT json_agg(json_build_object('conclusion', final_conclusion, 'count', count))
                 FROM conclusion_stats),
                '[]'::json
            ) as conclusions,
            COALESCE(
                (SELECT json_agg(json_build_object('category', category, 'count', count))
                 FROM category_stats),
                '[]'::json
            ) as categories
        "#
    )
    .bind(params.start_date.as_ref())
    .bind(params.end_date.as_ref());

    // Execute query with error handling and logging
    let row = match query_with_binds.fetch_one(&state.db).await {
        Ok(row) => row,
        Err(e) => {
            // Log the detailed error with proper tracing
            tracing::error!("❌ Error executing stats query: {:?}", e);

            // Check if it's a table missing error
            if e.to_string().contains("relation") && e.to_string().contains("does not exist") {
                tracing::warn!("⚠️  Sessions table does not exist. Database migrations may not have been run.");
                tracing::info!("💡 To create the sessions table, ensure DATABASE_URL is set and run database migrations.");
            }

            // Return empty stats gracefully instead of 500 error
            tracing::info!("📊 Returning empty stats due to query error (sessions table may be empty or missing)");
            return Ok(Json(DashboardStats {
                total_sessions: 0,
                completed_sessions: 0,
                abandoned_sessions: 0,
                active_sessions: 0,
                avg_steps_to_completion: 0.0,
                most_common_conclusions: vec![],
                sessions_by_category: vec![],
            }));
        }
    };

    let total_sessions: i64 = row.try_get("total_sessions").unwrap_or(0);
    let completed_sessions: i64 = row.try_get("completed_sessions").unwrap_or(0);
    let abandoned_sessions: i64 = row.try_get("abandoned_sessions").unwrap_or(0);
    let active_sessions: i64 = row.try_get("active_sessions").unwrap_or(0);
    let avg_steps_to_completion: f64 = row.try_get("avg_steps_to_completion").unwrap_or(0.0);

    // Debug logging to help diagnose avg_steps issues
    tracing::debug!(
        "📊 Stats: total={}, completed={}, abandoned={}, active={}, avg_steps={}",
        total_sessions, completed_sessions, abandoned_sessions, active_sessions, avg_steps_to_completion
    );

    if completed_sessions > 0 && avg_steps_to_completion == 0.0 {
        tracing::warn!(
            "⚠️  Avg steps is 0.0 but {} completed sessions exist. Check if 'steps' field is NULL/empty in database.",
            completed_sessions
        );
    }

    let conclusions_json: serde_json::Value = row.try_get("conclusions").unwrap_or(serde_json::json!([]));
    let most_common_conclusions: Vec<ConclusionStats> = serde_json::from_value(conclusions_json)
        .unwrap_or_default();

    let categories_json: serde_json::Value = row.try_get("categories").unwrap_or(serde_json::json!([]));
    let sessions_by_category: Vec<CategoryStats> = serde_json::from_value(categories_json)
        .unwrap_or_default();

    Ok(Json(DashboardStats {
        total_sessions,
        completed_sessions,
        abandoned_sessions,
        active_sessions,
        avg_steps_to_completion,
        most_common_conclusions,
        sessions_by_category,
    }))
}

/// GET /api/admin/audit-logs
/// Get audit logs (ADMIN only)
pub async fn get_audit_logs(_state: State<AppState>) -> ApiResult<Json<AuditLogsResponse>> {
    // Default pagination
    let page = 1;
    let page_size = 100;

    // TODO: Implement audit_logs table and query
    // For now, return empty response since audit_logs table doesn't exist yet
    Ok(Json(AuditLogsResponse {
        logs: vec![],
        total_count: 0,
        page,
        page_size,
    }))
}

/// Performance metrics response
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct PerformanceMetrics {
    pub database: DatabaseMetrics,
    pub cache: CacheMetrics,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct DatabaseMetrics {
    pub pool_size: u32,
    pub active_connections: usize,
    pub idle_connections: usize,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CacheMetrics {
    pub questions_cache: CacheStats,
    pub issue_tree_cache: CacheStats,
    pub issue_graph_cache: CacheStats,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub max_size: usize,
    pub ttl_seconds: u64,
}

/// GET /api/admin/performance
/// Get performance metrics (ADMIN only)
pub async fn get_performance_metrics(
    State(state): State<AppState>,
) -> ApiResult<Json<PerformanceMetrics>> {
    // Database connection pool metrics
    let pool_size = state.db.size();
    let idle_connections = state.db.num_idle();
    let active_connections = (pool_size as usize).saturating_sub(idle_connections);

    // Cache metrics
    let questions_stats = state.questions_cache.stats().await;
    let tree_stats = state.issue_tree_cache.stats().await;
    let graph_stats = state.issue_graph_cache.stats().await;

    Ok(Json(PerformanceMetrics {
        database: DatabaseMetrics {
            pool_size,
            active_connections,
            idle_connections,
        },
        cache: CacheMetrics {
            questions_cache: CacheStats {
                total_entries: questions_stats.total_entries,
                active_entries: questions_stats.active_entries,
                expired_entries: questions_stats.expired_entries,
                max_size: questions_stats.max_size,
                ttl_seconds: questions_stats.ttl_seconds,
            },
            issue_tree_cache: CacheStats {
                total_entries: tree_stats.total_entries,
                active_entries: tree_stats.active_entries,
                expired_entries: tree_stats.expired_entries,
                max_size: tree_stats.max_size,
                ttl_seconds: tree_stats.ttl_seconds,
            },
            issue_graph_cache: CacheStats {
                total_entries: graph_stats.total_entries,
                active_entries: graph_stats.active_entries,
                expired_entries: graph_stats.expired_entries,
                max_size: graph_stats.max_size,
                ttl_seconds: graph_stats.ttl_seconds,
            },
        },
    }))
}

/// DELETE /api/admin/sessions
/// Delete sessions based on filters (ADMIN only)
pub async fn delete_sessions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    Query(params): Query<DeleteSessionsParams>,
) -> ApiResult<Json<DeleteSessionsResponse>> {
    // Build DELETE query safely using QueryBuilder to prevent SQL injection
    use sqlx::QueryBuilder;
    let mut query = QueryBuilder::new("DELETE FROM sessions WHERE 1=1");

    // Time range filter based on started_at
    if let Some(time_range) = &params.time_range {
        match time_range.as_str() {
            "today" => {
                query.push(" AND started_at >= CURRENT_DATE");
            }
            "past_week" => {
                query.push(" AND started_at >= NOW() - INTERVAL '7 days'");
            }
            "past_month" => {
                query.push(" AND started_at >= NOW() - INTERVAL '30 days'");
            }
            "all_time" => {
                // No time filter, all sessions
            }
            _ => {
                tracing::warn!("Invalid time_range value: {}", time_range);
            }
        }
    }

    // Category filter (issue category) - SAFE: uses parameterized query
    if let Some(category) = &params.category {
        query.push(" AND (steps->0->>'category')::text = ");
        query.push_bind(category);
    }

    // Status filter
    if let Some(status) = &params.status {
        match status.as_str() {
            "completed" => {
                query.push(" AND completed_at IS NOT NULL");
            }
            "abandoned" => {
                query.push(" AND (abandoned = true OR (completed_at IS NULL AND started_at <= NOW() - INTERVAL '1 hour'))");
            }
            "active" => {
                query.push(" AND completed_at IS NULL");
                query.push(" AND abandoned = false");
                query.push(" AND started_at > NOW() - INTERVAL '1 hour'");
            }
            "all" => {
                // No status filter
            }
            _ => {
                tracing::warn!("Invalid status value: {}", status);
            }
        }
    }

    tracing::info!(
        "🗑️  Executing session deletion with filters - time_range: {:?}, category: {:?}, status: {:?}",
        params.time_range,
        params.category,
        params.status
    );

    let result = match query.build().execute(&state.db).await {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("❌ Error deleting sessions: {:?}", e);
            return Err(crate::error::ApiError::internal(
                "Failed to delete sessions",
            ));
        }
    };

    let deleted_count = result.rows_affected() as i64;

    tracing::info!("✅ Successfully deleted {} sessions", deleted_count);

    // Audit log the session deletion
    let user_id = Uuid::parse_str(&auth.0.sub)
        .map_err(|_| crate::error::ApiError::internal("Invalid user ID in token"))?;
    let ip = audit::extract_ip_address(&headers);

    audit::log_event(
        &state.db,
        user_id,
        audit::AuditAction::SessionsDeleted,
        "sessions",
        None,
        Some(json!({
            "deleted_count": deleted_count,
            "time_range": &params.time_range,
            "category": &params.category,
            "status": &params.status,
        })),
        ip.as_deref(),
    )
    .await?;

    Ok(Json(DeleteSessionsResponse { deleted_count }))
}

/// GET /api/admin/sessions/count
/// Get count of sessions matching filters (for preview before delete)
pub async fn count_sessions(
    State(state): State<AppState>,
    Query(params): Query<DeleteSessionsParams>,
) -> ApiResult<Json<serde_json::Value>> {
    // Build COUNT query safely using QueryBuilder to prevent SQL injection
    use sqlx::QueryBuilder;
    let mut query = QueryBuilder::new("SELECT COUNT(*) FROM sessions WHERE 1=1");

    // Time range filter
    if let Some(time_range) = &params.time_range {
        match time_range.as_str() {
            "today" => {
                query.push(" AND started_at >= CURRENT_DATE");
            }
            "past_week" => {
                query.push(" AND started_at >= NOW() - INTERVAL '7 days'");
            }
            "past_month" => {
                query.push(" AND started_at >= NOW() - INTERVAL '30 days'");
            }
            "all_time" => {}
            _ => {
                tracing::warn!("Invalid time_range value: {}", time_range);
            }
        }
    }

    // Category filter - SAFE: uses parameterized query
    if let Some(category) = &params.category {
        query.push(" AND (steps->0->>'category')::text = ");
        query.push_bind(category);
    }

    // Status filter
    if let Some(status) = &params.status {
        match status.as_str() {
            "completed" => {
                query.push(" AND completed_at IS NOT NULL");
            }
            "abandoned" => {
                query.push(" AND (abandoned = true OR (completed_at IS NULL AND started_at <= NOW() - INTERVAL '1 hour'))");
            }
            "active" => {
                query.push(" AND completed_at IS NULL");
                query.push(" AND abandoned = false");
                query.push(" AND started_at > NOW() - INTERVAL '1 hour'");
            }
            "all" => {}
            _ => {
                tracing::warn!("Invalid status value: {}", status);
            }
        }
    }

    let count = match query.build_query_scalar::<i64>()
        .fetch_one(&state.db)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("❌ Error counting sessions: {:?}", e);
            return Err(crate::error::ApiError::internal(
                "Failed to count sessions",
            ));
        }
    };

    Ok(Json(serde_json::json!({ "count": count })))
}

/// Response for listing categories
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CategoryListResponse {
    pub categories: Vec<String>,
}

/// Request for renaming a category
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct RenameCategoryRequest {
    pub new_name: String,
}

/// Response for category update operations
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CategoryUpdateResponse {
    pub updated_count: u64,
}

/// GET /api/admin/categories
/// List all unique display_category values
pub async fn list_categories(State(state): State<AppState>) -> ApiResult<Json<CategoryListResponse>> {
    let categories = sqlx::query!(
        r#"
        SELECT DISTINCT display_category
        FROM nodes
        WHERE display_category IS NOT NULL
        ORDER BY display_category ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let category_list: Vec<String> = categories
        .into_iter()
        .filter_map(|row| row.display_category)
        .collect();

    Ok(Json(CategoryListResponse {
        categories: category_list,
    }))
}

/// PUT /api/admin/categories/:name
/// Rename a category (updates all nodes using it)
pub async fn rename_category(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<RenameCategoryRequest>,
) -> ApiResult<Json<CategoryUpdateResponse>> {
    let result = sqlx::query!(
        r#"
        UPDATE nodes
        SET display_category = $1
        WHERE display_category = $2
        "#,
        req.new_name,
        name
    )
    .execute(&state.db)
    .await?;

    Ok(Json(CategoryUpdateResponse {
        updated_count: result.rows_affected(),
    }))
}

/// DELETE /api/admin/categories/:name
/// Delete a category by setting display_category to NULL for all nodes using it
pub async fn delete_category(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> ApiResult<Json<CategoryUpdateResponse>> {
    let result = sqlx::query!(
        r#"
        UPDATE nodes
        SET display_category = NULL
        WHERE display_category = $1
        "#,
        name
    )
    .execute(&state.db)
    .await?;

    Ok(Json(CategoryUpdateResponse {
        updated_count: result.rows_affected(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary() {
        let summary = SessionSummary {
            session_id: "test-123".to_string(),
            started_at: "2025-10-24T00:00:00Z".to_string(),
            completed_at: Some("2025-10-24T00:05:00Z".to_string()),
            abandoned: false,
            tech_identifier: Some("Tech123".to_string()),
            client_site: Some("Site A".to_string()),
            final_conclusion: Some("Test conclusion".to_string()),
            step_count: 5,
        };
        assert_eq!(summary.step_count, 5);
    }

    #[test]
    fn test_dashboard_stats() {
        let stats = DashboardStats {
            total_sessions: 100,
            completed_sessions: 80,
            abandoned_sessions: 15,
            active_sessions: 5,
            avg_steps_to_completion: 4.5,
            most_common_conclusions: vec![],
            sessions_by_category: vec![],
        };
        assert_eq!(stats.total_sessions, 100);
    }
}
