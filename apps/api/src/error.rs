use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// API Error types with TypeScript export
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../apps/web/src/types/")]
#[serde(tag = "type", content = "data")]
pub enum ApiError {
    /// Resource not found (404)
    NotFound { message: String },

    /// Unauthorized - missing or invalid authentication (401)
    Unauthorized { message: String },

    /// Forbidden - authenticated but insufficient permissions (403)
    Forbidden { message: String },

    /// Validation error with field-specific messages (422)
    ValidationError { fields: Vec<ValidationField> },

    /// Database error (500)
    DatabaseError { message: String },

    /// Internal server error (500)
    InternalError { message: String },

    /// Bad request - invalid input (400)
    BadRequest { message: String },

    /// Conflict - resource already exists (409)
    Conflict { message: String },
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../apps/web/src/types/")]
pub struct ValidationField {
    pub field: String,
    pub message: String,
}

/// Standard error response format
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../apps/web/src/types/")]
pub struct ErrorResponse {
    pub error: ApiError,
    pub timestamp: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::NotFound {
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        ApiError::Unauthorized {
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        ApiError::Forbidden {
            message: message.into(),
        }
    }

    pub fn validation(fields: Vec<(String, String)>) -> Self {
        ApiError::ValidationError {
            fields: fields
                .into_iter()
                .map(|(field, message)| ValidationField { field, message })
                .collect(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        ApiError::DatabaseError {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        ApiError::InternalError {
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::BadRequest {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        ApiError::Conflict {
            message: message.into(),
        }
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
        }
    }
}

/// Implement Axum's IntoResponse for automatic error handling
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        let error_response = ErrorResponse {
            error: self,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (status, Json(error_response)).into_response()
    }
}

/// Convert SQLx errors to API errors
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::not_found("Resource not found"),
            sqlx::Error::Database(db_err) => {
                tracing::error!("Database error: {}", db_err);
                ApiError::database("Database operation failed")
            }
            _ => {
                tracing::error!("SQLx error: {}", err);
                ApiError::internal("An unexpected error occurred")
            }
        }
    }
}

/// Convert serde_json errors to API errors
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON serialization error: {}", err);
        ApiError::internal("Data serialization error")
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::not_found("test").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::unauthorized("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::forbidden("test").status_code(),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn test_validation_error() {
        let error = ApiError::validation(vec![
            ("email".to_string(), "Invalid email format".to_string()),
            ("password".to_string(), "Password too short".to_string()),
        ]);

        match error {
            ApiError::ValidationError { fields } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].field, "email");
            }
            _ => panic!("Expected validation error"),
        }
    }
}
