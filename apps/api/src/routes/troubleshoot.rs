use crate::error::{ApiError, ApiResult};
use crate::models::{Answer, Question, Node, Connection, NodeType};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Request to start a new troubleshooting session
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct StartSessionRequest {
    pub tech_identifier: Option<String>,
    pub client_site: Option<String>,
    pub category: Option<String>, // Optional: for direct category access
}

/// Response when starting a session (NODE-GRAPH VERSION)
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct StartSessionResponse {
    pub session_id: String,
    pub node: Node,
    pub options: Vec<NavigationOption>,
}

/// Navigation option (connection to next node)
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct NavigationOption {
    pub connection_id: Uuid,
    pub label: String,
    pub target_category: String,
    pub display_category: Option<String>,
}

/// Request to submit an answer (NODE-GRAPH VERSION)
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct SubmitAnswerRequest {
    pub connection_id: Uuid,
}

/// Response after submitting an answer (NODE-GRAPH VERSION)
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct SubmitAnswerResponse {
    pub session_id: String,
    pub node: Node,
    pub options: Vec<NavigationOption>,
    pub is_conclusion: bool,
    pub conclusion_text: Option<String>,
}

/// A step in the troubleshooting session history
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct HistoryStep {
    pub question: Question,
    pub answer: Answer,
}

/// Response containing session history
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../../apps/web/src/types/")]
pub struct SessionHistoryResponse {
    pub session_id: String,
    pub started_at: String,
    pub completed: bool,
    pub steps: Vec<HistoryStep>,
    pub final_conclusion: Option<String>,
}

/// POST /api/troubleshoot/start
/// Start a new troubleshooting session (public) - NODE-GRAPH VERSION
pub async fn start_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<StartSessionRequest>,
) -> ApiResult<Json<StartSessionResponse>> {
    // Get the starting node based on category or default to global start
    let root_node = if let Some(category) = &req.category {
        // Direct category access: find the category's start node
        let semantic_id = format!("{}_start", category);
        sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE semantic_id = $1 AND is_active = true"
        )
        .bind(&semantic_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found(&format!("Issue category '{}' not found", category)))?
    } else {
        // No category specified: use global start node
        sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE semantic_id = 'start' AND is_active = true"
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::internal("Global start node not found. Please run ensure_global_start.sql"))?
    };

    // Get connections from root node
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = $1 AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(root_node.id)
    .fetch_all(&state.db)
    .await?;

    // For each connection, get the target node to build options
    let mut options = Vec::new();
    for conn in connections {
        let target_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE id = $1"
        )
        .bind(conn.to_node_id)
        .fetch_one(&state.db)
        .await?;

        options.push(NavigationOption {
            connection_id: conn.id,
            label: conn.label,
            target_category: target_node.category,
            display_category: target_node.display_category.clone(),
        });
    }

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();

    // Get user agent and IP for tracking
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    // Hash IP address for privacy (simple MD5 for now)
    let ip_hash = ip_address.map(|ip| format!("{:x}", md5::compute(ip.as_bytes())));

    // Create session in database
    let initial_steps = serde_json::json!([]);

    sqlx::query(
        "INSERT INTO sessions (session_id, started_at, steps, tech_identifier, client_site, user_agent, ip_hash, abandoned)
         VALUES ($1, NOW(), $2, $3, $4, $5, $6, false)",
    )
    .bind(&session_id)
    .bind(&initial_steps)
    .bind(&req.tech_identifier)
    .bind(&req.client_site)
    .bind(&user_agent)
    .bind(&ip_hash)
    .execute(&state.db)
    .await?;

    Ok(Json(StartSessionResponse {
        session_id,
        node: root_node,
        options,
    }))
}

/// POST /api/troubleshoot/:session_id/answer
/// Submit an answer and get the next node (public) - NODE-GRAPH VERSION
pub async fn submit_answer(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SubmitAnswerRequest>,
) -> ApiResult<Json<SubmitAnswerResponse>> {
    // Verify session exists and get current state
    let session = sqlx::query!(
        "SELECT id, steps, completed_at FROM sessions WHERE session_id = $1",
        session_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Session not found"))?;

    // Check if session is already completed
    if session.completed_at.is_some() {
        return Err(ApiError::bad_request("Session is already completed"));
    }

    // Get the connection
    let connection = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE id = $1 AND is_active = true"
    )
    .bind(req.connection_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    // Get the from_node (current node)
    let from_node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(connection.from_node_id)
    .fetch_one(&state.db)
    .await?;

    // Get the target node
    let next_node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(connection.to_node_id)
    .fetch_one(&state.db)
    .await?;

    // Update session steps
    let mut steps: Vec<serde_json::Value> = serde_json::from_value(session.steps.clone())
        .unwrap_or_default();

    steps.push(serde_json::json!({
        "node_id": from_node.id,
        "node_text": from_node.text,
        "connection_id": connection.id,
        "connection_label": connection.label,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }));

    let steps_json = serde_json::to_value(&steps)?;

    // Check if this is a conclusion node
    if matches!(next_node.node_type, NodeType::Conclusion) {
        // Session is complete
        sqlx::query(
            "UPDATE sessions
             SET steps = $1, final_conclusion = $2, completed_at = NOW(), abandoned = false
             WHERE session_id = $3"
        )
        .bind(&steps_json)
        .bind(&next_node.text)
        .bind(&session_id)
        .execute(&state.db)
        .await?;

        return Ok(Json(SubmitAnswerResponse {
            session_id,
            node: next_node.clone(),
            options: vec![],
            is_conclusion: true,
            conclusion_text: Some(next_node.text),
        }));
    }

    // Get next connections (if question node)
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = $1 AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(next_node.id)
    .fetch_all(&state.db)
    .await?;

    let mut options = Vec::new();
    for conn in connections {
        let target_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE id = $1"
        )
        .bind(conn.to_node_id)
        .fetch_one(&state.db)
        .await?;

        options.push(NavigationOption {
            connection_id: conn.id,
            label: conn.label,
            target_category: target_node.category,
            display_category: target_node.display_category.clone(),
        });
    }

    // Update session
    sqlx::query(
        "UPDATE sessions SET steps = $1 WHERE session_id = $2"
    )
    .bind(&steps_json)
    .bind(&session_id)
    .execute(&state.db)
    .await?;

    Ok(Json(SubmitAnswerResponse {
        session_id,
        node: next_node,
        options,
        is_conclusion: false,
        conclusion_text: None,
    }))
}

/// GET /api/troubleshoot/:session_id
/// Get current state of a session (public) - NODE-GRAPH VERSION
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<SubmitAnswerResponse>> {
    // Get session
    let session = sqlx::query!(
        "SELECT steps, final_conclusion, completed_at FROM sessions WHERE session_id = $1",
        session_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Session not found"))?;

    // Parse steps to find current position
    let steps: Vec<serde_json::Value> = serde_json::from_value(session.steps)
        .unwrap_or_default();

    // If no steps, return starting node
    if steps.is_empty() {
        let root_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE semantic_id = 'start' AND is_active = true"
        )
        .fetch_one(&state.db)
        .await?;

        let connections = sqlx::query_as::<_, Connection>(
            "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
             FROM connections
             WHERE from_node_id = $1 AND is_active = true
             ORDER BY order_index ASC"
        )
        .bind(root_node.id)
        .fetch_all(&state.db)
        .await?;

        let mut options = Vec::new();
        for conn in connections {
            let target_node = sqlx::query_as::<_, Node>(
                "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
                 FROM nodes
                 WHERE id = $1"
            )
            .bind(conn.to_node_id)
            .fetch_one(&state.db)
            .await?;

            options.push(NavigationOption {
                connection_id: conn.id,
                label: conn.label,
                target_category: target_node.category,
                display_category: target_node.display_category.clone(),
            });
        }

        return Ok(Json(SubmitAnswerResponse {
            session_id,
            node: root_node,
            options,
            is_conclusion: false,
            conclusion_text: None,
        }));
    }

    // Get last connection to determine current node
    let last_step = &steps[steps.len() - 1];
    let last_connection_id: Uuid = serde_json::from_value(last_step["connection_id"].clone())
        .map_err(|_| ApiError::internal("Invalid session data"))?;

    let last_connection = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE id = $1"
    )
    .bind(last_connection_id)
    .fetch_one(&state.db)
    .await?;

    // Get current node (target of last connection)
    let current_node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(last_connection.to_node_id)
    .fetch_one(&state.db)
    .await?;

    // If current node is a conclusion, session should be marked complete
    if matches!(current_node.node_type, NodeType::Conclusion) {
        return Ok(Json(SubmitAnswerResponse {
            session_id,
            node: current_node.clone(),
            options: vec![],
            is_conclusion: true,
            conclusion_text: Some(current_node.text),
        }));
    }

    // Get connections from current node
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = $1 AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(current_node.id)
    .fetch_all(&state.db)
    .await?;

    let mut options = Vec::new();
    for conn in connections {
        let target_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE id = $1"
        )
        .bind(conn.to_node_id)
        .fetch_one(&state.db)
        .await?;

        options.push(NavigationOption {
            connection_id: conn.id,
            label: conn.label,
            target_category: target_node.category,
            display_category: target_node.display_category.clone(),
        });
    }

    Ok(Json(SubmitAnswerResponse {
        session_id,
        node: current_node,
        options,
        is_conclusion: false,
        conclusion_text: None,
    }))
}

/// GET /api/troubleshoot/:session_id/history
/// Get the full history of a session (public)
pub async fn get_session_history(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<SessionHistoryResponse>> {
    // Get session
    let session = sqlx::query!(
        "SELECT started_at, completed_at, steps, final_conclusion FROM sessions WHERE session_id = $1",
        session_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Session not found"))?;

    // Parse steps
    let steps: Vec<serde_json::Value> = serde_json::from_value(session.steps)
        .unwrap_or_default();

    // Build history with full question and answer details
    let mut history = Vec::new();

    for step in steps {
        let question_id: Uuid = serde_json::from_value(step["question_id"].clone())
            .map_err(|_| ApiError::internal("Invalid session data"))?;
        let answer_id: Uuid = serde_json::from_value(step["answer_id"].clone())
            .map_err(|_| ApiError::internal("Invalid session data"))?;

        let question = sqlx::query_as::<_, Question>(
            "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
             FROM questions
             WHERE id = $1",
        )
        .bind(question_id)
        .fetch_one(&state.db)
        .await?;

        let answer = sqlx::query_as::<_, Answer>(
            "SELECT id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at
             FROM answers
             WHERE id = $1",
        )
        .bind(answer_id)
        .fetch_one(&state.db)
        .await?;

        history.push(HistoryStep { question, answer });
    }

    Ok(Json(SessionHistoryResponse {
        session_id,
        started_at: session.started_at.to_rfc3339(),
        completed: session.completed_at.is_some(),
        steps: history,
        final_conclusion: session.final_conclusion,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_session_request() {
        let req = StartSessionRequest {
            tech_identifier: Some("Tech123".to_string()),
            client_site: Some("Site A".to_string()),
            category: None,
        };
        assert!(req.tech_identifier.is_some());
    }

    #[test]
    fn test_submit_answer_request() {
        let req = SubmitAnswerRequest {
            connection_id: Uuid::new_v4(),
        };
        assert!(!req.connection_id.to_string().is_empty());
    }
}
