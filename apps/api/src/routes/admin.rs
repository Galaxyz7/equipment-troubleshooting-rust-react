use crate::error::ApiResult;
use crate::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ConclusionStats {
    pub conclusion: String,
    pub count: i64,
}

/// Statistics by category
#[derive(Debug, Serialize, TS)]
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

    // Get total count with same filters
    let count_query = format!("SELECT COUNT(*) FROM sessions {}", where_clause);
    let total_count = sqlx::query_scalar::<_, i64>(&count_query)
        .fetch_one(&state.db)
        .await?;

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

    let sessions = sqlx::query_as::<_, (
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
    .await?;

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
/// Get dashboard statistics (ADMIN only)
pub async fn get_stats(
    State(state): State<AppState>,
    Query(params): Query<StatsQueryParams>,
) -> ApiResult<Json<DashboardStats>> {
    // Build date filter if provided
    let date_filter = if params.start_date.is_some() || params.end_date.is_some() {
        let mut conditions = vec![];
        if let Some(start) = &params.start_date {
            conditions.push(format!("started_at >= '{}'", start));
        }
        if let Some(end) = &params.end_date {
            conditions.push(format!("started_at <= '{}'", end));
        }
        format!("WHERE {}", conditions.join(" AND "))
    } else {
        String::new()
    };

    // Total sessions
    let total_sessions = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM sessions {}",
        date_filter
    ))
    .fetch_one(&state.db)
    .await?;

    // Build WHERE clause for queries with additional conditions
    let completed_where = if date_filter.is_empty() {
        "WHERE completed_at IS NOT NULL".to_string()
    } else {
        format!("{} AND completed_at IS NOT NULL", date_filter)
    };

    let abandoned_where = if date_filter.is_empty() {
        "WHERE abandoned = true".to_string()
    } else {
        format!("{} AND abandoned = true", date_filter)
    };

    let active_where = if date_filter.is_empty() {
        "WHERE completed_at IS NULL AND abandoned = false".to_string()
    } else {
        format!(
            "{} AND completed_at IS NULL AND abandoned = false",
            date_filter
        )
    };

    let conclusions_where = if date_filter.is_empty() {
        "WHERE final_conclusion IS NOT NULL".to_string()
    } else {
        format!("{} AND final_conclusion IS NOT NULL", date_filter)
    };

    // Completed sessions
    let completed_sessions = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM sessions {}",
        completed_where
    ))
    .fetch_one(&state.db)
    .await?;

    // Abandoned sessions
    let abandoned_sessions = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM sessions {}",
        abandoned_where
    ))
    .fetch_one(&state.db)
    .await?;

    // Active sessions (started but not completed and not abandoned)
    let active_sessions = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM sessions {}",
        active_where
    ))
    .fetch_one(&state.db)
    .await?;

    // Average steps to completion
    let avg_steps: Option<f64> = sqlx::query_scalar(&format!(
        r#"
        SELECT CAST(AVG(jsonb_array_length(steps)) AS DOUBLE PRECISION)
        FROM sessions
        {}
        "#,
        completed_where
    ))
    .fetch_one(&state.db)
    .await?;

    let avg_steps_to_completion = avg_steps.unwrap_or(0.0);

    // Most common conclusions (top 10)
    let most_common_conclusions = sqlx::query_as::<_, (String, i64)>(&format!(
        r#"
        SELECT final_conclusion, COUNT(*) as count
        FROM sessions
        {}
        GROUP BY final_conclusion
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "#,
        conclusions_where
    ))
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(conclusion, count)| ConclusionStats { conclusion, count })
    .collect();

    // Sessions by category - extract from first step in steps JSONB array
    let sessions_by_category = sqlx::query_as::<_, (String, i64)>(&format!(
        r#"
        SELECT
            COALESCE((steps->0->>'category')::text, 'unknown') as category,
            COUNT(*) as count
        FROM sessions
        {}
        GROUP BY category
        ORDER BY count DESC
        "#,
        date_filter
    ))
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(category, count)| CategoryStats { category, count })
    .collect();

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
