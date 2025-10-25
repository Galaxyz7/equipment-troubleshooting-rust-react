use crate::error::ApiResult;
use crate::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Session summary for admin list view
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
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
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct SessionsListResponse {
    pub sessions: Vec<SessionSummary>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Query parameters for sessions list
#[derive(Debug, Deserialize)]
pub struct SessionsListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub completed: Option<bool>,
    pub abandoned: Option<bool>,
}

/// Dashboard statistics response
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
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
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct ConclusionStats {
    pub conclusion: String,
    pub count: i64,
}

/// Statistics by category
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct CategoryStats {
    pub category: String,
    pub count: i64,
}

/// Audit log entry
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
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
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLogEntry>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

/// GET /api/admin/sessions
/// List all sessions with pagination and filters (ADMIN only)
pub async fn list_sessions(
    State(state): State<AppState>,
    // In a real implementation, we'd extract query params from the request
    // For now, we'll use defaults
) -> ApiResult<Json<SessionsListResponse>> {
    // Default pagination
    let page = 1;
    let page_size = 50;
    let offset = (page - 1) * page_size;

    // Get total count
    let total_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(&state.db)
        .await?;

    // Fetch sessions with pagination
    let sessions = sqlx::query!(
        r#"
        SELECT
            session_id,
            started_at,
            completed_at,
            abandoned,
            tech_identifier,
            client_site,
            final_conclusion,
            COALESCE(jsonb_array_length(steps), 0)::int as "step_count!"
        FROM sessions
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
        page_size as i64,
        offset as i64
    )
    .fetch_all(&state.db)
    .await?;

    let session_summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|s| SessionSummary {
            session_id: s.session_id,
            started_at: s.started_at.to_rfc3339(),
            completed_at: s.completed_at.map(|dt| dt.to_rfc3339()),
            abandoned: s.abandoned,
            tech_identifier: s.tech_identifier,
            client_site: s.client_site,
            final_conclusion: s.final_conclusion,
            step_count: s.step_count,
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
pub async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<DashboardStats>> {
    // Total sessions
    let total_sessions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(&state.db)
        .await?;

    // Completed sessions
    let completed_sessions =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE completed_at IS NOT NULL")
            .fetch_one(&state.db)
            .await?;

    // Abandoned sessions
    let abandoned_sessions =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE abandoned = true")
            .fetch_one(&state.db)
            .await?;

    // Active sessions (started but not completed and not abandoned)
    let active_sessions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions WHERE completed_at IS NULL AND abandoned = false",
    )
    .fetch_one(&state.db)
    .await?;

    // Average steps to completion
    let avg_steps: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT CAST(AVG(jsonb_array_length(steps)) AS DOUBLE PRECISION)
        FROM sessions
        WHERE completed_at IS NOT NULL
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let avg_steps_to_completion = avg_steps.unwrap_or(0.0);

    // Most common conclusions (top 10)
    let most_common_conclusions = sqlx::query!(
        r#"
        SELECT final_conclusion as "conclusion!", COUNT(*) as "count!"
        FROM sessions
        WHERE final_conclusion IS NOT NULL
        GROUP BY final_conclusion
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|row| ConclusionStats {
        conclusion: row.conclusion,
        count: row.count,
    })
    .collect();

    // Sessions by category - simplified for now (returns empty)
    // TODO: Implement category extraction by joining with questions table
    let sessions_by_category: Vec<CategoryStats> = vec![];

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
