use crate::error::{ApiError, ApiResult};
use crate::models::{Node, CreateNode, UpdateNode, NodeType, NodeWithConnections, ConnectionWithTarget};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    pub category: Option<String>,
    pub node_type: Option<String>,
}

/// GET /api/nodes
/// List all nodes, optionally filtered by category or type
pub async fn list_nodes(
    State(state): State<AppState>,
    Query(query): Query<ListNodesQuery>,
) -> ApiResult<Json<Vec<Node>>> {
    let mut sql = String::from(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE is_active = true"
    );

    let mut conditions = Vec::new();

    if let Some(ref category) = query.category {
        conditions.push(format!("category = '{}'", category));
    }

    if let Some(ref node_type) = query.node_type {
        conditions.push(format!("node_type = '{}'", node_type));
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at ASC");

    let nodes = sqlx::query_as::<_, Node>(&sql)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(nodes))
}

/// GET /api/nodes/:id
/// Get a specific node by ID
pub async fn get_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Node>> {
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Node not found"))?;

    Ok(Json(node))
}

/// POST /api/nodes
/// Create a new node (ADMIN only)
pub async fn create_node(
    State(state): State<AppState>,
    Json(req): Json<CreateNode>,
) -> ApiResult<Json<Node>> {
    // Validate input
    if req.text.is_empty() {
        return Err(ApiError::validation(vec![(
            "text".to_string(),
            "Node text is required".to_string(),
        )]));
    }

    // Insert node
    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (category, node_type, text, semantic_id, display_category, position_x, position_y, is_active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true)
         RETURNING id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at"
    )
    .bind(&req.category)
    .bind(&req.node_type)
    .bind(&req.text)
    .bind(&req.semantic_id)
    .bind(&req.display_category)
    .bind(&req.position_x)
    .bind(&req.position_y)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(node))
}

/// PUT /api/nodes/:id
/// Update a node (ADMIN only)
pub async fn update_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNode>,
) -> ApiResult<Json<Node>> {
    // Check if node exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(ApiError::not_found("Node not found"));
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE nodes SET updated_at = NOW()");
    let mut param_count = 1;

    if req.text.is_some() {
        param_count += 1;
        query.push_str(&format!(", text = ${}", param_count));
    }
    if req.semantic_id.is_some() {
        param_count += 1;
        query.push_str(&format!(", semantic_id = ${}", param_count));
    }
    if req.node_type.is_some() {
        param_count += 1;
        query.push_str(&format!(", node_type = ${}", param_count));
    }
    if req.display_category.is_some() {
        param_count += 1;
        query.push_str(&format!(", display_category = ${}", param_count));
    }
    if req.position_x.is_some() {
        param_count += 1;
        query.push_str(&format!(", position_x = ${}", param_count));
    }
    if req.position_y.is_some() {
        param_count += 1;
        query.push_str(&format!(", position_y = ${}", param_count));
    }
    if req.is_active.is_some() {
        param_count += 1;
        query.push_str(&format!(", is_active = ${}", param_count));
    }

    query.push_str(" WHERE id = $1 RETURNING id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Node>(&query).bind(id);

    if let Some(text) = req.text {
        query_builder = query_builder.bind(text);
    }
    if let Some(semantic_id) = req.semantic_id {
        query_builder = query_builder.bind(semantic_id);
    }
    if let Some(node_type) = req.node_type {
        query_builder = query_builder.bind(node_type);
    }
    if let Some(display_category) = req.display_category {
        query_builder = query_builder.bind(display_category);
    }
    if let Some(position_x) = req.position_x {
        query_builder = query_builder.bind(position_x);
    }
    if let Some(position_y) = req.position_y {
        query_builder = query_builder.bind(position_y);
    }
    if let Some(is_active) = req.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let node = query_builder.fetch_one(&state.db).await?;

    Ok(Json(node))
}

/// DELETE /api/nodes/:id
/// Soft delete a node (ADMIN only)
pub async fn delete_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Node>> {
    let node = sqlx::query_as::<_, Node>(
        "UPDATE nodes
         SET is_active = false, updated_at = NOW()
         WHERE id = $1
         RETURNING id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Node not found"))?;

    Ok(Json(node))
}

/// GET /api/nodes/:id/with-connections
/// Get a node with all its outgoing connections and target node details
pub async fn get_node_with_connections(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<NodeWithConnections>> {
    // Get the node
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, display_category, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Node not found"))?;

    // Get connections with target nodes
    let connections_with_targets = sqlx::query!(
        r#"
        SELECT
            c.id,
            c.label,
            c.order_index,
            n.id as target_id,
            n.category as target_category,
            n.node_type as target_node_type,
            n.text as target_text,
            n.semantic_id as target_semantic_id,
            n.display_category as target_display_category,
            n.position_x as target_position_x,
            n.position_y as target_position_y,
            n.is_active as target_is_active,
            n.created_at as target_created_at,
            n.updated_at as target_updated_at
        FROM connections c
        JOIN nodes n ON c.to_node_id = n.id
        WHERE c.from_node_id = $1 AND c.is_active = true
        ORDER BY c.order_index ASC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    let connections = connections_with_targets
        .into_iter()
        .map(|row| {
            ConnectionWithTarget {
                id: row.id,
                label: row.label,
                order_index: row.order_index.unwrap_or(0),
                target_node: Node {
                    id: row.target_id,
                    category: row.target_category,
                    node_type: match row.target_node_type.as_str() {
                        "question" => NodeType::Question,
                        "conclusion" => NodeType::Conclusion,
                        _ => NodeType::Question,
                    },
                    text: row.target_text,
                    semantic_id: row.target_semantic_id,
                    display_category: row.target_display_category,
                    position_x: row.target_position_x,
                    position_y: row.target_position_y,
                    is_active: row.target_is_active.unwrap_or(true),
                    created_at: row.target_created_at.unwrap_or_else(chrono::Utc::now),
                    updated_at: row.target_updated_at.unwrap_or_else(chrono::Utc::now),
                },
            }
        })
        .collect();

    Ok(Json(NodeWithConnections {
        node,
        connections,
    }))
}
