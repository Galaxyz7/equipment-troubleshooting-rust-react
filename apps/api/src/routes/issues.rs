use crate::error::{ApiError, ApiResult};
use crate::models::{Answer, Question, Node, Connection, IssueGraph};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
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

/// Tree node representing a question and its branches
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct TreeNode {
    pub question: Question,
    pub answers: Vec<TreeAnswer>,
}

/// Answer with its destination information
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct TreeAnswer {
    pub id: String,
    pub label: String,
    pub order_index: i32,
    pub destination: AnswerDestination,
}

/// Where an answer leads to
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct AnswerDestination {
    #[serde(rename = "type")]
    pub destination_type: String, // "question" or "conclusion"
    pub question_id: Option<String>,
    pub question_text: Option<String>,
    pub conclusion_text: Option<String>,
}

/// Complete tree structure for an issue
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct IssueTree {
    pub issue: Issue,
    pub nodes: Vec<TreeNode>,
}

// ============================================
// ROUTE HANDLERS
// ============================================

/// GET /api/admin/issues
/// List all issues (categories with root nodes) - NODE-GRAPH VERSION
pub async fn list_issues(State(state): State<AppState>) -> ApiResult<Json<Vec<Issue>>> {
    let issues = sqlx::query!(
        r#"
        SELECT DISTINCT ON (category)
            n.id,
            COALESCE(n.category, 'uncategorized') as "category!",
            COALESCE(n.category, 'Uncategorized') as "name!",
            n.display_category,
            n.id as root_node_id,
            n.is_active,
            n.created_at,
            n.updated_at,
            (SELECT COUNT(*) FROM nodes n2 WHERE n2.category = n.category OR (n2.category IS NULL AND n.category IS NULL)) as "question_count!"
        FROM nodes n
        ORDER BY category, n.created_at ASC
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

/// GET /api/admin/issues/:category/tree
/// Get complete decision tree for an issue - Cached for 10 minutes, optimized queries
pub async fn get_issue_tree(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> ApiResult<Json<IssueTree>> {
    // Try to get from cache first
    if let Some(cached) = state.issue_tree_cache.get(&category).await {
        tracing::debug!("✅ Cache HIT: issue tree for {}", category);
        return Ok(Json(serde_json::from_value(cached)?));
    }

    tracing::debug!("❌ Cache MISS: issue tree for {} - fetching from DB", category);

    // Get all questions in this category
    let questions = sqlx::query_as::<_, Question>(
        "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
         FROM questions
         WHERE category = $1
         ORDER BY created_at ASC",
    )
    .bind(&category)
    .fetch_all(&state.db)
    .await?;

    if questions.is_empty() {
        return Err(ApiError::not_found("Issue not found"));
    }

    // Get issue metadata from first question
    let first_question = &questions[0];
    let issue = Issue {
        id: first_question.id.to_string(),
        name: category.clone(),
        category: category.clone(),
        display_category: None,
        root_question_id: first_question.id.to_string(),
        is_active: first_question.is_active,
        question_count: questions.len() as i64,
        created_at: first_question.created_at.to_rfc3339(),
        updated_at: first_question.updated_at.to_rfc3339(),
    };

    // OPTIMIZATION: Fetch ALL answers for this category in one query instead of N queries
    let question_ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();
    let all_answers = sqlx::query_as::<_, Answer>(
        "SELECT id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at
         FROM answers
         WHERE question_id = ANY($1)
         ORDER BY question_id, order_index ASC",
    )
    .bind(&question_ids)
    .fetch_all(&state.db)
    .await?;

    // OPTIMIZATION: Fetch ALL referenced questions in one query instead of N queries
    let next_question_ids: Vec<Uuid> = all_answers
        .iter()
        .filter_map(|a| a.next_question_id)
        .collect();

    let next_questions = if !next_question_ids.is_empty() {
        sqlx::query_as::<_, Question>(
            "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
             FROM questions
             WHERE id = ANY($1)",
        )
        .bind(&next_question_ids)
        .fetch_all(&state.db)
        .await?
    } else {
        vec![]
    };

    // Create lookup map for quick access
    let question_map: std::collections::HashMap<Uuid, &Question> =
        next_questions.iter().map(|q| (q.id, q)).collect();

    // Build tree nodes
    let mut nodes = Vec::new();
    let mut answer_index = 0;

    for question in questions {
        let mut tree_answers = Vec::new();

        // Get answers for this question from the fetched batch
        while answer_index < all_answers.len()
            && all_answers[answer_index].question_id == question.id
        {
            let answer = &all_answers[answer_index];

            let destination = if let Some(next_q_id) = answer.next_question_id {
                AnswerDestination {
                    destination_type: "question".to_string(),
                    question_id: Some(next_q_id.to_string()),
                    question_text: question_map.get(&next_q_id).map(|q| q.text.clone()),
                    conclusion_text: None,
                }
            } else {
                AnswerDestination {
                    destination_type: "conclusion".to_string(),
                    question_id: None,
                    question_text: None,
                    conclusion_text: answer.conclusion_text.clone(),
                }
            };

            tree_answers.push(TreeAnswer {
                id: answer.id.to_string(),
                label: answer.label.clone(),
                order_index: answer.order_index,
                destination,
            });

            answer_index += 1;
        }

        nodes.push(TreeNode {
            question,
            answers: tree_answers,
        });
    }

    let result = IssueTree { issue, nodes };

    // Store in cache
    state.issue_tree_cache.set(category.clone(), serde_json::to_value(&result)?).await;

    Ok(Json(result))
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
         VALUES ($1, $2, 'question', $3, $4, $5, true)
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

    Ok(Json(Issue {
        id: node.id.to_string(),
        name: req.name.unwrap_or(category.clone()),
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

/// DELETE /api/admin/issues/:category
/// Delete entire issue and all its nodes/connections (NODE-GRAPH VERSION)
pub async fn delete_issue(
    State(state): State<AppState>,
    Path(category): Path<String>,
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

    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": result.rows_affected(),
        "message": format!("Issue '{}' deleted successfully", category)
    })))
}
