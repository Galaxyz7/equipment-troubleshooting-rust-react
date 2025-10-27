use equipment_troubleshooting::error::{ApiError, ApiResult};
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tokio::test]
async fn test_api_error_not_found() {
    let error = ApiError::not_found("Resource not found");
    assert_eq!(error.status_code(), StatusCode::NOT_FOUND);

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_error_unauthorized() {
    let error = ApiError::unauthorized("Invalid credentials");
    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_error_forbidden() {
    let error = ApiError::forbidden("Access denied");
    assert_eq!(error.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_api_error_bad_request() {
    let error = ApiError::bad_request("Invalid input");
    assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_error_internal() {
    let error = ApiError::internal("Server error");
    assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_api_error_validation() {
    let validation_errors = vec![
        ("email".to_string(), "Invalid email format".to_string()),
        ("password".to_string(), "Password too short".to_string()),
    ];

    let error = ApiError::validation(validation_errors);
    assert_eq!(error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// Test removed: conflict() method was unused and has been removed from the codebase
// The Conflict variant still exists in ApiError enum for future use

#[tokio::test]
async fn test_api_result_ok() {
    let result: ApiResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_api_result_err() {
    let result: ApiResult<i32> = Err(ApiError::not_found("Not found"));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_response_format() {
    use axum::response::IntoResponse;

    let error = ApiError::bad_request("Test error message");
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_multiple_validation_errors() {
    let errors = vec![
        ("field1".to_string(), "Error 1".to_string()),
        ("field2".to_string(), "Error 2".to_string()),
        ("field3".to_string(), "Error 3".to_string()),
    ];

    let error = ApiError::validation(errors.clone());
    assert_eq!(error.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_error_chaining() {
    // Test that we can create an error and convert it to response
    let error = ApiError::not_found("User not found");
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_from_sqlx_error() {
    // Test conversion from sqlx::Error to ApiError
    use sqlx::Error as SqlxError;

    let sqlx_error = SqlxError::RowNotFound;
    let api_error: ApiError = sqlx_error.into();
    assert_eq!(api_error.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_error_debug() {
    let error = ApiError::bad_request("Test message");
    let error_string = format!("{:?}", error);
    assert!(!error_string.is_empty());
}
