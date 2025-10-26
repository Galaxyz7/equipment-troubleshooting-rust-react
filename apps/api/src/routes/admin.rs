use crate::error::ApiResult;
use crate::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use ts_rs::TS;

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
    pub total_sessions: i64,
    pub completed_sessions: i64,
    pub abandoned_sessions: i64,
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
    pub count: i64,
}

/// Statistics by category
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CategoryStats {
    pub category: String,
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
    pub category: Option<String>,
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

    // Build WHERE clause dynamically based on filters
    let mut conditions: Vec<String> = vec![];

    if let Some(status) = &params.status {
        match status.as_str() {
            "completed" => conditions.push("completed_at IS NOT NULL".to_string()),
            "abandoned" => conditions.push("abandoned = true".to_string()),
            "active" => {
                conditions.push("completed_at IS NULL".to_string());
                conditions.push("abandoned = false".to_string());
            }
            _ => {}
        }
    }

    if let Some(start_date) = &params.start_date {
        conditions.push(format!("started_at >= '{}'", start_date));
    }

    if let Some(end_date) = &params.end_date {
        conditions.push(format!("started_at <= '{}'", end_date));
    }

    if let Some(search) = &params.search {
        conditions.push(format!(
            "(tech_identifier ILIKE '%{}%' OR client_site ILIKE '%{}%')",
            search.replace("'", "''"),
            search.replace("'", "''")
        ));
    }

    if let Some(category) = &params.category {
        // Filter by category in the first step
        conditions.push(format!(
            "(steps->0->>'category')::text = '{}'",
            category.replace("'", "''")
        ));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Get total count with same filters (with error handling)
    let count_query = format!("SELECT COUNT(*) FROM sessions {}", where_clause);
    let total_count = match sqlx::query_scalar::<_, i64>(&count_query)
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

    // Fetch sessions with pagination and filters
    let query = format!(
        r#"
        SELECT
            session_id,
            started_at,
            completed_at,
            abandoned,
            tech_identifier,
            client_site,
            final_conclusion,
            COALESCE(jsonb_array_length(steps), 0)::int as step_count
        FROM sessions
        {}
        ORDER BY started_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, page_size, offset
    );

    let sessions = match sqlx::query_as::<_, (
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
        bool,
        Option<String>,
        Option<String>,
        Option<String>,
        i32,
    )>(&query)
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
    // Build date filter conditions
    let mut date_conditions = vec![];
    if let Some(start) = &params.start_date {
        date_conditions.push(format!("started_at >= '{}'", start));
    }
    if let Some(end) = &params.end_date {
        date_conditions.push(format!("started_at <= '{}'", end));
    }
    let date_filter = if date_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", date_conditions.join(" AND "))
    };

    // Single optimized query using CTEs to compute all stats in one database roundtrip
    let query = format!(
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
            {}
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
        "#,
        date_filter
    );

    // Execute query with error handling and logging
    let row = match sqlx::query(&query).fetch_one(&state.db).await {
        Ok(row) => row,
        Err(e) => {
            // Log the detailed error with proper tracing
            tracing::error!("❌ Error executing stats query: {:?}", e);
            tracing::debug!("SQL Query that failed: {}", query);

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
