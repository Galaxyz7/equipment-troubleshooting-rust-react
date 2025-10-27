use crate::error::{ApiError, ApiResult};
use crate::middleware::auth::AuthUser;
use crate::models::{Node, Connection, IssueGraph, NodeType};
use crate::utils::audit;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;
use uuid::Uuid;

// ============================================
// TYPES & MODELS
// ============================================

/// Issue represents a top-level troubleshooting category
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Issue {
    pub id: String,
    pub name: String,
    pub category: String,
    pub display_category: Option<String>,
    pub root_question_id: String,
    pub is_active: bool,
    pub question_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create a new issue
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CreateIssueRequest {
    pub name: String,
    pub category: String,
    pub display_category: Option<String>,
    pub root_question_text: String,
}

/// Request to update issue metadata
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UpdateIssueRequest {
    pub name: Option<String>,
    pub display_category: Option<String>,
    pub is_active: Option<bool>,
}

/// Query parameters for toggle_issue
#[derive(Debug, Deserialize)]
pub struct ToggleIssueQuery {
    #[serde(default)]
    pub force: bool,
}

// ============================================
// IMPORT/EXPORT TYPES
// ============================================

/// Export data for a single issue (used for backup/restore)
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct IssueExportData {
    /// Issue metadata for import
    pub issue: IssueImportMetadata,
    /// All nodes in this issue category
    pub nodes: Vec<NodeExportData>,
    /// All connections between nodes
    pub connections: Vec<ConnectionExportData>,
}

/// Issue metadata for import (without generated fields)
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct IssueImportMetadata {
    pub name: String,
    pub category: String,
    pub display_category: Option<String>,
    pub root_question_text: String,
}

/// Node data for export (with index references instead of UUIDs)
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct NodeExportData {
    pub node_type: String, // "Question" or "Conclusion"
    pub text: String,
    pub semantic_id: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

/// Connection data for export (with node array indices instead of UUIDs)
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ConnectionExportData {
    /// Index in nodes array (not UUID)
    pub from_node_index: usize,
    /// Index in nodes array (not UUID)
    pub to_node_index: usize,
    pub label: String,
    pub order_index: i32,
}

/// Result of importing issues
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ImportResult {
    pub success: Vec<ImportSuccess>,
    pub errors: Vec<ImportError>,
}

/// Successfully imported issue
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ImportSuccess {
    pub category: String,
    pub name: String,
    pub nodes_count: usize,
    pub connections_count: usize,
}

/// Error during import
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ImportError {
    pub category: String,
    pub error: String,
}

// ============================================
// ROUTE HANDLERS
// ============================================

/// GET /api/admin/issues
/// List all issues (categories with root nodes) - NODE-GRAPH VERSION
pub async fn list_issues(State(state): State<AppState>) -> ApiResult<Json<Vec<Issue>>> {
    let issues = sqlx::query!(
        r#"
        SELECT DISTINCT ON (n.category)
            n.id,
            COALESCE(n.category, 'uncategorized') as "category!",
            COALESCE(c.label, n.category, 'Uncategorized') as "name!",
            n.display_category,
            n.id as root_node_id,
            n.is_active,
            n.created_at,
            n.updated_at,
            (SELECT COUNT(*) FROM nodes n2 WHERE n2.category = n.category OR (n2.category IS NULL AND n.category IS NULL)) as "question_count!"
        FROM nodes n
        LEFT JOIN connections c ON c.to_node_id = n.id AND c.from_node_id = (SELECT id FROM nodes WHERE semantic_id = 'start' LIMIT 1)
        ORDER BY n.category, n.created_at ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let issue_list = issues
        .into_iter()
        .map(|row| Issue {
            id: row.id.to_string(),
            name: row.name,
            category: row.category,
            display_category: row.display_category,
            root_question_id: row.root_node_id.to_string(),
            is_active: row.is_active.unwrap_or(true),
            question_count: row.question_count,
            created_at: row.created_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
            updated_at: row.updated_at.unwrap_or_else(chrono::Utc::now).to_rfc3339(),
        })
        .collect();

    Ok(Json(issue_list))
}

/// GET /api/admin/issues/:category/graph
/// Get complete node graph for an issue category - Cached for 10 minutes
pub async fn get_issue_graph(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> ApiResult<Json<IssueGraph>> {
    // Try to get from cache first
    let cache_key = format!("graph_{}", category);
    if let Some(cached) = state.issue_graph_cache.get(&cache_key).await {
        tracing::debug!("✅ Cache HIT: issue graph for {}", category);
        return Ok(Json(serde_json::from_value(cached)?));
    }

    tracing::debug!("❌ Cache MISS: issue graph for {} - fetching from DB", category);

    // Get all active nodes in this category
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE category = $1 AND is_active = true
         ORDER BY created_at ASC"
    )
    .bind(&category)
    .fetch_all(&state.db)
    .await?;

    if nodes.is_empty() {
        return Err(ApiError::not_found("Issue category not found"));
    }

    // Get all node IDs
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Get all active connections between these nodes
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = ANY($1) AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(&node_ids)
    .fetch_all(&state.db)
    .await?;

    let result = IssueGraph {
        category: category.clone(),
        nodes,
        connections,
    };

    // Store in cache
    state.issue_graph_cache.set(cache_key, serde_json::to_value(&result)?).await;

    Ok(Json(result))
}

/// POST /api/admin/issues
/// Create a new issue with root node (NODE-GRAPH VERSION)
pub async fn create_issue(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    Json(req): Json<CreateIssueRequest>,
) -> ApiResult<Json<Issue>> {
    // Start a transaction for atomicity and use a single optimized query
    let mut tx = state.db.begin().await?;

    // Validate category is unique
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE category = $1 LIMIT 1)"
    )
    .bind(&req.category)
    .fetch_one(&mut *tx)
    .await?;

    if existing {
        return Err(ApiError::validation(vec![(
            "category".to_string(),
            "Category already exists".to_string(),
        )]));
    }

    // Create root node for this issue category and return it in one query
    let node_id = Uuid::new_v4();
    let semantic_id = format!("{}_start", req.category);

    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (id, category, node_type, text, semantic_id, display_category, is_active)
         VALUES ($1, $2, 'question', $3, $4, $5, false)
         RETURNING id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at"
    )
    .bind(node_id)
    .bind(&req.category)
    .bind(&req.root_question_text)
    .bind(&semantic_id)
    .bind(req.display_category.as_deref())
    .fetch_one(&mut *tx)
    .await?;

    // Automatically link this new issue to the root node (semantic_id='start')
    // Use a single query with a subquery for order_index
    sqlx::query!(
        r#"
        INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active)
        SELECT
            n.id,
            $1,
            $2,
            COALESCE((SELECT COUNT(*) FROM connections WHERE from_node_id = n.id), 0)::int,
            true
        FROM nodes n
        WHERE n.semantic_id = 'start'
        "#,
        node_id,
        &req.name
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Audit log the issue creation
    let user_id = Uuid::parse_str(&auth.0.sub)
        .map_err(|_| ApiError::internal("Invalid user ID in token"))?;
    let ip = audit::extract_ip_address(&headers);

    audit::log_event(
        &state.db,
        user_id,
        audit::AuditAction::IssueCreated,
        "issue",
        Some(&req.category),
        Some(json!({
            "name": &req.name,
            "display_category": &node.display_category,
            "root_question_text": &req.root_question_text,
        })),
        ip.as_deref(),
    )
    .await?;

    Ok(Json(Issue {
        id: node.id.to_string(),
        name: req.name,
        category: req.category,
        display_category: node.display_category,
        root_question_id: node.id.to_string(),
        is_active: node.is_active,
        question_count: 1,
        created_at: node.created_at.to_rfc3339(),
        updated_at: node.updated_at.to_rfc3339(),
    }))
}

/// PUT /api/admin/issues/:category
/// Update issue metadata (NODE-GRAPH VERSION)
pub async fn update_issue(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    Path(category): Path<String>,
    Json(req): Json<UpdateIssueRequest>,
) -> ApiResult<Json<Issue>> {
    // Check if issue exists
    let mut node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE category = $1
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&category)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Issue not found"))?;

    // Variable to track the updated name
    let updated_name = if let Some(name) = &req.name {
        // Update the connection label (where the issue name is actually stored)
        // The connection goes from the 'start' node to this issue's root node
        sqlx::query!(
            r#"
            UPDATE connections
            SET label = $1
            WHERE to_node_id = $2
              AND from_node_id = (SELECT id FROM nodes WHERE semantic_id = 'start' LIMIT 1)
            "#,
            name,
            node.id
        )
        .execute(&state.db)
        .await?;
        name.clone()
    } else {
        // Fetch current name from connection label
        let conn = sqlx::query!(
            r#"
            SELECT label
            FROM connections
            WHERE to_node_id = $1
              AND from_node_id = (SELECT id FROM nodes WHERE semantic_id = 'start' LIMIT 1)
            "#,
            node.id
        )
        .fetch_optional(&state.db)
        .await?;
        conn.map(|c| c.label).unwrap_or_else(|| category.clone())
    };

    // Update display_category if provided
    if let Some(display_category) = &req.display_category {
        // Update all nodes in this category
        sqlx::query!(
            "UPDATE nodes SET display_category = $1 WHERE category = $2",
            display_category.as_str(),
            &category
        )
        .execute(&state.db)
        .await?;
        node.display_category = Some(display_category.clone());
    }

    // Update is_active status if provided
    if let Some(is_active) = req.is_active {
        // Update all nodes in this category
        sqlx::query!(
            "UPDATE nodes SET is_active = $1 WHERE category = $2",
            is_active,
            &category
        )
        .execute(&state.db)
        .await?;
        node.is_active = is_active;
    }

    // Get updated count
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM nodes WHERE category = $1",
        &category
    )
    .fetch_one(&state.db)
    .await?;

    // Audit log the issue update
    let user_id = Uuid::parse_str(&auth.0.sub)
        .map_err(|_| ApiError::internal("Invalid user ID in token"))?;
    let ip = audit::extract_ip_address(&headers);

    audit::log_event(
        &state.db,
        user_id,
        audit::AuditAction::IssueUpdated,
        "issue",
        Some(&category),
        Some(json!({
            "name": req.name,
            "display_category": req.display_category,
            "is_active": req.is_active,
        })),
        ip.as_deref(),
    )
    .await?;

    Ok(Json(Issue {
        id: node.id.to_string(),
        name: updated_name,
        category: category.clone(),
        display_category: node.display_category,
        root_question_id: node.id.to_string(),
        is_active: node.is_active,
        question_count: count.count.unwrap_or(0),
        created_at: node.created_at.to_rfc3339(),
        updated_at: node.updated_at.to_rfc3339(),
    }))
}

/// PATCH /api/admin/issues/:category/toggle
/// Toggle issue active status (NODE-GRAPH VERSION)
pub async fn toggle_issue(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    Path(category): Path<String>,
    Query(query): Query<ToggleIssueQuery>,
) -> ApiResult<Json<Issue>> {
    // Get current status and root node
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE category = $1
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&category)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Issue not found"))?;

    let new_status = !node.is_active;

    // If activating (turning on) and not forced, validate for incomplete nodes
    if new_status && !query.force {
        // Find all Question nodes in this category that have no outgoing connections
        let incomplete_nodes = sqlx::query!(
            r#"
            SELECT n.id, n.text, n.semantic_id
            FROM nodes n
            WHERE n.category = $1
            AND n.node_type = 'Question'
            AND NOT EXISTS (
                SELECT 1 FROM connections c
                WHERE c.from_node_id = n.id
            )
            "#,
            &category
        )
        .fetch_all(&state.db)
        .await?;

        if !incomplete_nodes.is_empty() {
            // Build error message with node details
            let node_details: Vec<String> = incomplete_nodes
                .iter()
                .map(|n| {
                    format!("{} ({})", n.text, n.semantic_id.as_ref().unwrap_or(&"no ID".to_string()))
                })
                .collect();

            return Err(ApiError::validation(vec![(
                "incomplete_nodes".to_string(),
                format!(
                    "This issue has {} end node(s) with no conclusion: {}. These nodes need outgoing connections or should be changed to Conclusion type.",
                    incomplete_nodes.len(),
                    node_details.join(", ")
                ),
            )]));
        }
    }

    // Toggle all nodes in this category
    sqlx::query!(
        "UPDATE nodes SET is_active = $1 WHERE category = $2",
        new_status,
        &category
    )
    .execute(&state.db)
    .await?;

    // IMPORTANT: Also toggle any connections that point to this category's root node
    // This ensures that when you toggle "Brush" off, the "Brush" connection in the root node also gets disabled
    sqlx::query!(
        "UPDATE connections SET is_active = $1 WHERE to_node_id = $2",
        new_status,
        node.id
    )
    .execute(&state.db)
    .await?;

    // Get count
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM nodes WHERE category = $1",
        &category
    )
    .fetch_one(&state.db)
    .await?;

    // Audit log the issue toggle
    let user_id = Uuid::parse_str(&auth.0.sub)
        .map_err(|_| ApiError::internal("Invalid user ID in token"))?;
    let ip = audit::extract_ip_address(&headers);

    audit::log_event(
        &state.db,
        user_id,
        audit::AuditAction::IssueToggled,
        "issue",
        Some(&category),
        Some(json!({
            "new_status": new_status,
            "forced": query.force,
        })),
        ip.as_deref(),
    )
    .await?;

    Ok(Json(Issue {
        id: node.id.to_string(),
        name: category.clone(),
        category: category.clone(),
        display_category: node.display_category,
        root_question_id: node.id.to_string(),
        is_active: new_status,
        question_count: count.count.unwrap_or(0),
        created_at: node.created_at.to_rfc3339(),
        updated_at: node.updated_at.to_rfc3339(),
    }))
}

/// Query parameters for delete issue endpoint
#[derive(Debug, serde::Deserialize)]
pub struct DeleteIssueParams {
    #[serde(default)]
    pub delete_sessions: bool,
}

/// DELETE /api/admin/issues/:category
/// Delete entire issue and all its nodes/connections (NODE-GRAPH VERSION)
pub async fn delete_issue(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    headers: HeaderMap,
    Path(category): Path<String>,
    Query(params): Query<DeleteIssueParams>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check if issue exists
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM nodes WHERE category = $1",
        &category
    )
    .fetch_one(&state.db)
    .await?;

    if count.count.unwrap_or(0) == 0 {
        return Err(ApiError::not_found("Issue not found"));
    }

    // Delete all connections for nodes in this category
    // (Note: cascade delete will handle this automatically if FK constraints are set up,
    // but doing it explicitly for clarity)
    sqlx::query!(
        "DELETE FROM connections WHERE from_node_id IN (SELECT id FROM nodes WHERE category = $1)",
        &category
    )
    .execute(&state.db)
    .await?;

    // Delete all nodes in this category
    let result = sqlx::query!(
        "DELETE FROM nodes WHERE category = $1",
        &category
    )
    .execute(&state.db)
    .await?;

    let nodes_deleted = result.rows_affected();

    // Optionally delete all sessions associated with this category
    let sessions_deleted = if params.delete_sessions {
        let sessions_result = sqlx::query(
            "DELETE FROM sessions WHERE (steps->0->>'category')::text = $1"
        )
        .bind(&category)
        .execute(&state.db)
        .await?;

        let count = sessions_result.rows_affected();
        tracing::info!("🗑️  Deleted {} sessions for category '{}'", count, category);
        count
    } else {
        0
    };

    // Audit log the issue deletion
    let user_id = Uuid::parse_str(&auth.0.sub)
        .map_err(|_| ApiError::internal("Invalid user ID in token"))?;
    let ip = audit::extract_ip_address(&headers);

    audit::log_event(
        &state.db,
        user_id,
        audit::AuditAction::IssueDeleted,
        "issue",
        Some(&category),
        Some(json!({
            "nodes_deleted": nodes_deleted,
            "sessions_deleted": sessions_deleted,
            "delete_sessions": params.delete_sessions,
        })),
        ip.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": nodes_deleted,
        "sessions_deleted": sessions_deleted,
        "message": format!("Issue '{}' deleted successfully", category)
    })))
}

// ============================================
// IMPORT/EXPORT ENDPOINTS
// ============================================

/// GET /api/admin/issues/:category/export
/// Export a single issue with all its nodes and connections as JSON
pub async fn export_issue(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> ApiResult<Json<IssueExportData>> {
    tracing::info!("📦 Exporting issue: {}", category);

    // Get all nodes for this category
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE category = $1 AND is_active = true
         ORDER BY created_at ASC"
    )
    .bind(&category)
    .fetch_all(&state.db)
    .await?;

    if nodes.is_empty() {
        return Err(ApiError::not_found("Issue category not found"));
    }

    // Build ID to index mapping (use UUID as key)
    let mut id_to_index = std::collections::HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        id_to_index.insert(node.id, index);
    }

    // Get the root node to extract issue metadata
    let root_node = nodes.iter().find(|n| n.semantic_id.as_ref().map(|s| s.ends_with("_start")).unwrap_or(false))
        .ok_or_else(|| ApiError::not_found("Root node not found for issue"))?;

    // Get issue name from database (try to find it via display_category or use category)
    let issue_name = root_node.display_category.clone().unwrap_or_else(|| category.clone());

    // Export nodes (without UUIDs)
    let export_nodes: Vec<NodeExportData> = nodes.iter().map(|n| NodeExportData {
        node_type: match n.node_type {
            NodeType::Question => "question".to_string(),
            NodeType::Conclusion => "conclusion".to_string(),
        },
        text: n.text.clone(),
        semantic_id: n.semantic_id.clone(),
        position_x: n.position_x,
        position_y: n.position_y,
    }).collect();

    // Get all node IDs for connection query
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Get all connections
    let connections = if !node_ids.is_empty() {
        sqlx::query_as::<_, Connection>(
            "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
             FROM connections
             WHERE from_node_id = ANY($1) AND is_active = true
             ORDER BY from_node_id, order_index ASC"
        )
        .bind(&node_ids)
        .fetch_all(&state.db)
        .await?
    } else {
        vec![]
    };

    // Export connections (with indices instead of UUIDs)
    let export_connections: Vec<ConnectionExportData> = connections.iter().filter_map(|c| {
        let from_index = id_to_index.get(&c.from_node_id)?;
        let to_index = id_to_index.get(&c.to_node_id)?;
        Some(ConnectionExportData {
            from_node_index: *from_index,
            to_node_index: *to_index,
            label: c.label.clone(),
            order_index: c.order_index,
        })
    }).collect();

    let export_data = IssueExportData {
        issue: IssueImportMetadata {
            name: issue_name,
            category: category.clone(),
            display_category: root_node.display_category.clone(),
            root_question_text: root_node.text.clone(),
        },
        nodes: export_nodes,
        connections: export_connections,
    };

    tracing::info!("✅ Exported issue {} ({} nodes, {} connections)", category, nodes.len(), connections.len());

    Ok(Json(export_data))
}

/// GET /api/admin/issues/export-all
/// Export all issues as a JSON array
pub async fn export_all_issues(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<IssueExportData>>> {
    tracing::info!("📦 Exporting all issues");

    // Get all distinct categories (excluding 'root' and utility categories)
    let categories: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT category FROM nodes
         WHERE category NOT IN ('root', 'electrical', 'general', 'mechanical')
         AND is_active = true
         ORDER BY category ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let mut all_exports = Vec::new();

    for category in categories {
        // Reuse the single export logic
        match export_issue(State(state.clone()), Path(category.clone())).await {
            Ok(Json(export_data)) => all_exports.push(export_data),
            Err(e) => {
                tracing::warn!("⚠️  Failed to export issue {}: {:?}", category, e);
                continue;
            }
        }
    }

    tracing::info!("✅ Exported {} issues", all_exports.len());

    Ok(Json(all_exports))
}

/// POST /api/admin/issues/import
/// Import one or more issues from JSON
pub async fn import_issues(
    State(state): State<AppState>,
    Json(data): Json<Vec<IssueExportData>>,
) -> ApiResult<Json<ImportResult>> {
    tracing::info!("📥 Importing {} issue(s)", data.len());

    let mut success_list = Vec::new();
    let mut error_list = Vec::new();

    for issue_data in data {
        let category = issue_data.issue.category.clone();

        // Check if category already exists
        let existing_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM nodes WHERE category = $1"
        )
        .bind(&category)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

        if existing_count > 0 {
            error_list.push(ImportError {
                category: category.clone(),
                error: format!("Issue with category '{}' already exists. Please delete it first or choose a different category.", category),
            });
            continue;
        }

        // Start transaction for atomicity
        let mut tx = match state.db.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error_list.push(ImportError {
                    category: category.clone(),
                    error: format!("Failed to start transaction: {}", e),
                });
                continue;
            }
        };

        // Validate nodes
        if issue_data.nodes.is_empty() {
            error_list.push(ImportError {
                category: category.clone(),
                error: "Issue must have at least one node".to_string(),
            });
            continue;
        }

        // Create nodes and build mapping
        let mut node_ids = Vec::new();
        let mut error_msg: Option<String> = None;

        for node_data in &issue_data.nodes {
            let node_id = Uuid::new_v4();
            let node_type = node_data.node_type.as_str();

            // Validate node_type (lowercase as per model definition)
            if node_type != "question" && node_type != "conclusion" {
                error_msg = Some(format!("Invalid node_type: '{}'. Must be 'question' or 'conclusion'", node_type));
                break;
            }

            match sqlx::query!(
                "INSERT INTO nodes (id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)",
                node_id,
                &category,
                node_type,
                &node_data.text,
                node_data.semantic_id.as_deref(),
                issue_data.issue.display_category.as_deref(),
                node_data.position_x,
                node_data.position_y,
            )
            .execute(&mut *tx)
            .await {
                Ok(_) => node_ids.push(node_id),
                Err(e) => {
                    error_msg = Some(format!("Failed to create node: {}", e));
                    break;
                }
            }
        }

        // If there was an error, rollback and continue to next issue
        if let Some(err) = error_msg {
            let _ = tx.rollback().await;
            error_list.push(ImportError {
                category: category.clone(),
                error: err,
            });
            continue;
        }

        // Create connections
        let mut connections_created = 0;
        let mut conn_error_msg: Option<String> = None;

        for conn_data in &issue_data.connections {
            // Validate indices
            if conn_data.from_node_index >= node_ids.len() || conn_data.to_node_index >= node_ids.len() {
                conn_error_msg = Some("Invalid connection: node index out of bounds".to_string());
                break;
            }

            let from_id = node_ids[conn_data.from_node_index];
            let to_id = node_ids[conn_data.to_node_index];

            match sqlx::query!(
                "INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active)
                 VALUES ($1, $2, $3, $4, true)",
                from_id,
                to_id,
                &conn_data.label,
                conn_data.order_index,
            )
            .execute(&mut *tx)
            .await {
                Ok(_) => connections_created += 1,
                Err(e) => {
                    conn_error_msg = Some(format!("Failed to create connection: {}", e));
                    break;
                }
            }
        }

        // If there was a connection error, rollback and continue to next issue
        if let Some(err) = conn_error_msg {
            let _ = tx.rollback().await;
            error_list.push(ImportError {
                category: category.clone(),
                error: err,
            });
            continue;
        }

        // Commit transaction
        match tx.commit().await {
            Ok(_) => {
                success_list.push(ImportSuccess {
                    category: category.clone(),
                    name: issue_data.issue.name.clone(),
                    nodes_count: node_ids.len(),
                    connections_count: connections_created,
                });
                tracing::info!("✅ Imported issue: {} ({} nodes, {} connections)",
                    category, node_ids.len(), connections_created);
            }
            Err(e) => {
                error_list.push(ImportError {
                    category: category.clone(),
                    error: format!("Failed to commit transaction: {}", e),
                });
            }
        }
    }

    tracing::info!("📥 Import complete: {} succeeded, {} failed", success_list.len(), error_list.len());

    Ok(Json(ImportResult {
        success: success_list,
        errors: error_list,
    }))
}
