use crate::error::{ApiError, ApiResult};
use crate::models::{Answer, CreateAnswer, UpdateAnswer};
use crate::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

/// GET /api/questions/:question_id/answers
/// List all active answers for a question (public)
pub async fn list_answers(
    State(state): State<AppState>,
    Path(question_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Answer>>> {
    // Verify question exists
    let question_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1)",
    )
    .bind(question_id)
    .fetch_one(&state.db)
    .await?;

    if !question_exists {
        return Err(ApiError::not_found("Question not found"));
    }

    // Fetch answers
    let answers = sqlx::query_as::<_, Answer>(
        "SELECT id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at
         FROM answers
         WHERE question_id = $1 AND is_active = true
         ORDER BY order_index ASC",
    )
    .bind(question_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(answers))
}

/// POST /api/questions/:question_id/answers
/// Create new answer for a question (ADMIN only - middleware handles auth)
pub async fn create_answer(
    State(state): State<AppState>,
    Path(question_id): Path<Uuid>,
    Json(mut req): Json<CreateAnswer>,
) -> ApiResult<Json<Answer>> {
    // Override question_id from path (security: don't trust client)
    req.question_id = question_id;

    // Validate input
    if req.label.is_empty() {
        return Err(ApiError::validation(vec![(
            "label".to_string(),
            "Answer label is required".to_string(),
        )]));
    }

    // Verify question exists
    let question_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1)",
    )
    .bind(question_id)
    .fetch_one(&state.db)
    .await?;

    if !question_exists {
        return Err(ApiError::not_found("Question not found"));
    }

    // If next_question_id provided, verify it exists
    if let Some(next_q_id) = req.next_question_id {
        let next_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1)",
        )
        .bind(next_q_id)
        .fetch_one(&state.db)
        .await?;

        if !next_exists {
            return Err(ApiError::validation(vec![(
                "next_question_id".to_string(),
                "Referenced question does not exist".to_string(),
            )]));
        }
    }

    // Validate that both next_question_id and conclusion_text are not set at the same time
    // Allow placeholder answers with neither (can be linked later)
    let has_next = req.next_question_id.is_some();
    let has_conclusion = req.conclusion_text.as_ref().map_or(false, |s| !s.is_empty());

    if has_next && has_conclusion {
        return Err(ApiError::validation(vec![(
            "next_question_id".to_string(),
            "Cannot have both next_question_id and conclusion_text".to_string(),
        )]));
    }

    // Insert answer
    let answer = sqlx::query_as::<_, Answer>(
        "INSERT INTO answers (question_id, label, next_question_id, conclusion_text, order_index, is_active)
         VALUES ($1, $2, $3, $4, $5, true)
         RETURNING id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at",
    )
    .bind(&req.question_id)
    .bind(&req.label)
    .bind(&req.next_question_id)
    .bind(&req.conclusion_text)
    .bind(req.order_index)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(answer))
}

/// PUT /api/answers/:id
/// Update answer (ADMIN only - middleware handles auth)
pub async fn update_answer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAnswer>,
) -> ApiResult<Json<Answer>> {
    // Check if answer exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM answers WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(ApiError::not_found("Answer not found"));
    }

    // Validate label if provided
    if let Some(ref label) = req.label {
        if label.is_empty() {
            return Err(ApiError::validation(vec![(
                "label".to_string(),
                "Answer label cannot be empty".to_string(),
            )]));
        }
    }

    // If next_question_id provided, verify it exists
    if let Some(next_q_id) = req.next_question_id {
        let next_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1)",
        )
        .bind(next_q_id)
        .fetch_one(&state.db)
        .await?;

        if !next_exists {
            return Err(ApiError::validation(vec![(
                "next_question_id".to_string(),
                "Referenced question does not exist".to_string(),
            )]));
        }
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE answers SET updated_at = NOW()");
    let mut param_count = 1;

    if req.label.is_some() {
        param_count += 1;
        query.push_str(&format!(", label = ${}", param_count));
    }
    if req.next_question_id.is_some() {
        param_count += 1;
        query.push_str(&format!(", next_question_id = ${}", param_count));
    }
    if req.conclusion_text.is_some() {
        param_count += 1;
        query.push_str(&format!(", conclusion_text = ${}", param_count));
    }
    if req.order_index.is_some() {
        param_count += 1;
        query.push_str(&format!(", order_index = ${}", param_count));
    }
    if req.is_active.is_some() {
        param_count += 1;
        query.push_str(&format!(", is_active = ${}", param_count));
    }

    query.push_str(" WHERE id = $1 RETURNING id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Answer>(&query).bind(id);

    if let Some(label) = req.label {
        query_builder = query_builder.bind(label);
    }
    if let Some(next_question_id) = req.next_question_id {
        query_builder = query_builder.bind(next_question_id);
    }
    if let Some(conclusion_text) = req.conclusion_text {
        query_builder = query_builder.bind(conclusion_text);
    }
    if let Some(order_index) = req.order_index {
        query_builder = query_builder.bind(order_index);
    }
    if let Some(is_active) = req.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let answer = query_builder.fetch_one(&state.db).await?;

    Ok(Json(answer))
}

/// DELETE /api/answers/:id
/// Soft delete answer (ADMIN only - middleware handles auth)
pub async fn delete_answer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Answer>> {
    // Soft delete by setting is_active = false
    let answer = sqlx::query_as::<_, Answer>(
        "UPDATE answers
         SET is_active = false, updated_at = NOW()
         WHERE id = $1
         RETURNING id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Answer not found"))?;

    Ok(Json(answer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_answer_validation() {
        let req = CreateAnswer {
            question_id: Uuid::new_v4(),
            label: "Test Answer".to_string(),
            next_question_id: Some(Uuid::new_v4()),
            conclusion_text: None,
            order_index: 0,
        };
        assert!(!req.label.is_empty());
        assert!(req.next_question_id.is_some());
    }

    #[test]
    fn test_update_answer_partial() {
        let req = UpdateAnswer {
            label: Some("Updated label".to_string()),
            next_question_id: None,
            conclusion_text: None,
            order_index: Some(5),
            is_active: None,
        };
        assert!(req.label.is_some());
        assert!(req.order_index.is_some());
    }
}
