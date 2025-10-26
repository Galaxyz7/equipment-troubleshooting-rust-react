use crate::error::{ApiError, ApiResult};
use crate::middleware::auth::AuthUser;
use crate::models::{User, UserRole};
use crate::utils::jwt::{generate_token, generate_token_with_expiration, verify_token};
use crate::AppState;
use argon2::PasswordVerifier;
use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Login request payload
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    /// If true, token will not expire. If false, token expires in 15 minutes.
    #[serde(default)]
    pub remember_me: bool,
}

/// Login response with JWT token and user info
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// User information returned in login response
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: UserRole,
}

/// POST /api/auth/login
/// Authenticate user with email and password
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    // Validate input
    if req.email.is_empty() {
        return Err(ApiError::validation(vec![(
            "email".to_string(),
            "Email is required".to_string(),
        )]));
    }

    if req.password.is_empty() {
        return Err(ApiError::validation(vec![(
            "password".to_string(),
            "Password is required".to_string(),
        )]));
    }

    // Query user from database
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, is_active, created_at, updated_at
         FROM users
         WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::unauthorized("Invalid email or password"))?;

    // Check if user is active
    if !user.is_active {
        return Err(ApiError::forbidden("Account is disabled"));
    }

    // Verify password with Argon2
    let password_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|_| ApiError::internal("Invalid password hash format"))?;

    argon2::Argon2::default()
        .verify_password(req.password.as_bytes(), &password_hash)
        .map_err(|_| ApiError::unauthorized("Invalid email or password"))?;

    // Generate JWT token with appropriate expiration
    // If remember_me is true: token valid for 30 days (43200 minutes)
    // If remember_me is false: token valid for 15 minutes
    let token = if req.remember_me {
        tracing::info!("🔐 Login with 'stay signed in' enabled for user: {}", user.email);
        generate_token_with_expiration(user.id, user.email.clone(), user.role.clone(), 43200)?
    } else {
        tracing::info!("🔐 Login with short-lived session (15 min) for user: {}", user.email);
        generate_token_with_expiration(user.id, user.email.clone(), user.role.clone(), 15)?
    };

    // Return response
    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id.to_string(),
            email: user.email,
            role: user.role,
        },
    }))
}

/// Refresh token request payload
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct RefreshRequest {
    pub token: String,
}

/// POST /api/auth/refresh
/// Refresh a JWT token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<LoginResponse>> {
    // Verify the current token
    let claims = verify_token(&req.token)?;

    // Look up user to ensure they still exist and are active
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, is_active, created_at, updated_at
         FROM users
         WHERE id = $1"
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).map_err(|_| ApiError::unauthorized("Invalid token"))?)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    // Check if user is still active
    if !user.is_active {
        return Err(ApiError::forbidden("Account is disabled"));
    }

    // Generate new token
    let new_token = generate_token(user.id, user.email.clone(), user.role.clone())?;

    // Return response with new token
    Ok(Json(LoginResponse {
        token: new_token,
        user: UserInfo {
            id: user.id.to_string(),
            email: user.email,
            role: user.role,
        },
    }))
}

/// GET /api/auth/me
/// Get current user information (requires authentication)
pub async fn me(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
) -> ApiResult<Json<UserInfo>> {
    // Look up user from database to get latest info
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, role, is_active, created_at, updated_at
         FROM users
         WHERE id = $1"
    )
    .bind(uuid::Uuid::parse_str(&auth_user.0.sub).map_err(|_| ApiError::internal("Invalid user ID"))?)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    // Check if user is still active
    if !user.is_active {
        return Err(ApiError::forbidden("Account is disabled"));
    }

    Ok(Json(UserInfo {
        id: user.id.to_string(),
        email: user.email,
        role: user.role,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_validation() {
        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            remember_me: false,
        };
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.password, "password123");
    }

    #[test]
    fn test_user_info_serialization() {
        let user_info = UserInfo {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::Admin,
        };

        let json = serde_json::to_string(&user_info).unwrap();
        assert!(json.contains("test@example.com"));
    }
}
