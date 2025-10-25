use crate::error::{ApiError, ApiResult};
use crate::models::{Connection, CreateConnection, UpdateConnection};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListConnectionsQuery {
    pub from_node_id: Option<Uuid>,
    pub to_node_id: Option<Uuid>,
}

/// GET /api/connections
/// List connections, optionally filtered by from/to node
pub async fn list_connections(
    State(state): State<AppState>,
    Query(query): Query<ListConnectionsQuery>,
) -> ApiResult<Json<Vec<Connection>>> {
    let mut sql = String::from(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE is_active = true"
    );

    let mut conditions = Vec::new();

    if let Some(from_id) = query.from_node_id {
        conditions.push(format!("from_node_id = '{}'", from_id));
    }

    if let Some(to_id) = query.to_node_id {
        conditions.push(format!("to_node_id = '{}'", to_id));
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY order_index ASC");

    let connections = sqlx::query_as::<_, Connection>(&sql)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(connections))
}

/// POST /api/connections
/// Create new connection (ADMIN only)
pub async fn create_connection(
    State(state): State<AppState>,
    Json(req): Json<CreateConnection>,
) -> ApiResult<Json<Connection>> {
    // Validate both nodes exist
    let from_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
    )
    .bind(req.from_node_id)
    .fetch_one(&state.db)
    .await?;

    let to_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
    )
    .bind(req.to_node_id)
    .fetch_one(&state.db)
    .await?;

    if !from_exists || !to_exists {
        return Err(ApiError::validation(vec![(
            "nodes".to_string(),
            "One or both nodes do not exist".to_string(),
        )]));
    }

    // Validate label is not empty
    if req.label.is_empty() {
        return Err(ApiError::validation(vec![(
            "label".to_string(),
            "Connection label is required".to_string(),
        )]));
    }

    // Insert connection
    let connection = sqlx::query_as::<_, Connection>(
        "INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active)
         VALUES ($1, $2, $3, $4, true)
         RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at"
    )
    .bind(req.from_node_id)
    .bind(req.to_node_id)
    .bind(&req.label)
    .bind(req.order_index)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(connection))
}

/// PUT /api/connections/:id
/// Update connection (ADMIN only)
pub async fn update_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnection>,
) -> ApiResult<Json<Connection>> {
    // Check if connection exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM connections WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Err(ApiError::not_found("Connection not found"));
    }

    // If changing to_node_id, validate it exists
    if let Some(to_node_id) = req.to_node_id {
        let node_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
        )
        .bind(to_node_id)
        .fetch_one(&state.db)
        .await?;

        if !node_exists {
            return Err(ApiError::validation(vec![(
                "to_node_id".to_string(),
                "Target node does not exist".to_string(),
            )]));
        }
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE connections SET updated_at = NOW()");
    let mut param_count = 1;

    if req.to_node_id.is_some() {
        param_count += 1;
        query.push_str(&format!(", to_node_id = ${}", param_count));
    }
    if req.label.is_some() {
        param_count += 1;
        query.push_str(&format!(", label = ${}", param_count));
    }
    if req.order_index.is_some() {
        param_count += 1;
        query.push_str(&format!(", order_index = ${}", param_count));
    }
    if req.is_active.is_some() {
        param_count += 1;
        query.push_str(&format!(", is_active = ${}", param_count));
    }

    query.push_str(" WHERE id = $1 RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Connection>(&query).bind(id);

    if let Some(to_node_id) = req.to_node_id {
        query_builder = query_builder.bind(to_node_id);
    }
    if let Some(label) = req.label {
        query_builder = query_builder.bind(label);
    }
    if let Some(order_index) = req.order_index {
        query_builder = query_builder.bind(order_index);
    }
    if let Some(is_active) = req.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let connection = query_builder.fetch_one(&state.db).await?;

    Ok(Json(connection))
}

/// DELETE /api/connections/:id
/// Soft delete connection (ADMIN only)
pub async fn delete_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Connection>> {
    let connection = sqlx::query_as::<_, Connection>(
        "UPDATE connections
         SET is_active = false, updated_at = NOW()
         WHERE id = $1
         RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    Ok(Json(connection))
}
