use crate::error::{ApiError, ApiResult};
use crate::models::{Answer, CreateQuestion, Question, QuestionWithAnswers, UpdateQuestion};
use crate::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

/// GET /api/questions
/// List all active questions (public) - Cached for 5 minutes
pub async fn list_questions(State(state): State<AppState>) -> ApiResult<Json<Vec<Question>>> {
    const CACHE_KEY: &str = "active_questions";

    // Try to get from cache first
    if let Some(cached) = state.questions_cache.get(&CACHE_KEY.to_string()).await {
        tracing::debug!("✅ Cache HIT: questions list");
        return Ok(Json(serde_json::from_value(cached)?));
    }

    // Cache miss - fetch from database
    tracing::debug!("❌ Cache MISS: questions list - fetching from DB");
    let questions = sqlx::query_as::<_, Question>(
        "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
         FROM questions
         WHERE is_active = true
         ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    // Store in cache
    state.questions_cache.set(CACHE_KEY.to_string(), serde_json::to_value(&questions)?).await;

    Ok(Json(questions))
}

/// GET /api/questions/:id
/// Get single question with answers (public)
pub async fn get_question(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<QuestionWithAnswers>> {
    // Fetch question
    let question = sqlx::query_as::<_, Question>(
        "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
         FROM questions
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Question not found"))?;

    // Fetch answers for this question
    let answers = sqlx::query_as::<_, Answer>(
        "SELECT id, question_id, label, next_question_id, conclusion_text, order_index, is_active, created_at, updated_at
         FROM answers
         WHERE question_id = $1 AND is_active = true
         ORDER BY order_index ASC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(QuestionWithAnswers {
        id: question.id,
        semantic_id: question.semantic_id,
        text: question.text,
        category: question.category,
        answers,
    }))
}

/// POST /api/questions
/// Create new question (ADMIN only - middleware handles auth)
pub async fn create_question(
    State(state): State<AppState>,
    Json(req): Json<CreateQuestion>,
) -> ApiResult<Json<Question>> {
    // Validate input
    if req.semantic_id.is_empty() {
        return Err(ApiError::validation(vec![(
            "semantic_id".to_string(),
            "Semantic ID is required".to_string(),
        )]));
    }

    if req.text.is_empty() {
        return Err(ApiError::validation(vec![(
            "text".to_string(),
            "Question text is required".to_string(),
        )]));
    }

    // Check for duplicate semantic_id
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM questions WHERE semantic_id = $1)",
    )
    .bind(&req.semantic_id)
    .fetch_one(&state.db)
    .await?;

    if existing {
        return Err(ApiError::conflict("A question with this semantic ID already exists"));
    }

    // Insert question
    let question = sqlx::query_as::<_, Question>(
        "INSERT INTO questions (semantic_id, text, category, is_active)
         VALUES ($1, $2, $3, true)
         RETURNING id, semantic_id, text, category, is_active, created_at, updated_at",
    )
    .bind(&req.semantic_id)
    .bind(&req.text)
    .bind(&req.category)
    .fetch_one(&state.db)
    .await?;

    // Invalidate questions cache
    state.questions_cache.invalidate(&"active_questions".to_string()).await;

    Ok(Json(question))
}

/// PUT /api/questions/:id
/// Update question (ADMIN only - middleware handles auth)
pub async fn update_question(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateQuestion>,
) -> ApiResult<Json<Question>> {
    // Check if question exists
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(ApiError::not_found("Question not found"));
    }

    // Validate text if provided
    if let Some(ref text) = req.text {
        if text.is_empty() {
            return Err(ApiError::validation(vec![(
                "text".to_string(),
                "Question text cannot be empty".to_string(),
            )]));
        }
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE questions SET updated_at = NOW()");
    let mut param_count = 1;

    if req.text.is_some() {
        param_count += 1;
        query.push_str(&format!(", text = ${}", param_count));
    }
    if req.category.is_some() {
        param_count += 1;
        query.push_str(&format!(", category = ${}", param_count));
    }
    if req.is_active.is_some() {
        param_count += 1;
        query.push_str(&format!(", is_active = ${}", param_count));
    }

    query.push_str(" WHERE id = $1 RETURNING id, semantic_id, text, category, is_active, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Question>(&query).bind(id);

    if let Some(text) = req.text {
        query_builder = query_builder.bind(text);
    }
    if let Some(category) = req.category {
        query_builder = query_builder.bind(category);
    }
    if let Some(is_active) = req.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let question = query_builder.fetch_one(&state.db).await?;

    // Invalidate questions cache
    state.questions_cache.invalidate(&"active_questions".to_string()).await;

    Ok(Json(question))
}

/// DELETE /api/questions/:id
/// Hard delete question and all its answers (ADMIN only - middleware handles auth)
pub async fn delete_question(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Question>> {
    // Fetch the question first to return it after deletion
    let question = sqlx::query_as::<_, Question>(
        "SELECT id, semantic_id, text, category, is_active, created_at, updated_at
         FROM questions
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Question not found"))?;

    // Delete all answers for this question
    sqlx::query("DELETE FROM answers WHERE question_id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    // Delete the question itself
    sqlx::query("DELETE FROM questions WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    // Invalidate questions cache
    state.questions_cache.invalidate(&"active_questions".to_string()).await;

    Ok(Json(question))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_question_validation() {
        let req = CreateQuestion {
            semantic_id: "test".to_string(),
            text: "Is this a test?".to_string(),
            category: Some("test".to_string()),
        };
        assert!(!req.semantic_id.is_empty());
        assert!(!req.text.is_empty());
    }

    #[test]
    fn test_update_question_partial() {
        let req = UpdateQuestion {
            text: Some("Updated text".to_string()),
            category: None,
            is_active: None,
        };
        assert!(req.text.is_some());
        assert!(req.category.is_none());
    }
}
