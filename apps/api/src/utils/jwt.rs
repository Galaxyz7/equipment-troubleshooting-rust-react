use crate::error::{ApiError, ApiResult};
use crate::models::UserRole;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Claims {
    /// User ID
    pub sub: String, // subject (user id)
    /// User email
    pub email: String,
    /// User role
    pub role: UserRole,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: Uuid, email: String, role: UserRole) -> Self {
        let now = Utc::now();
        let expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(24);

        Self {
            sub: user_id.to_string(),
            email,
            role,
            iat: now.timestamp(),
            exp: (now + Duration::hours(expiration_hours)).timestamp(),
        }
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Generate JWT token for user
pub fn generate_token(user_id: Uuid, email: String, role: UserRole) -> ApiResult<String> {
    let claims = Claims::new(user_id, email, role);

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ApiError::internal("JWT_SECRET not configured"))?;

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate JWT: {}", e);
        ApiError::internal("Failed to generate authentication token")
    })?;

    Ok(token)
}

/// Verify and decode JWT token
pub fn verify_token(token: &str) -> ApiResult<Claims> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ApiError::internal("JWT_SECRET not configured"))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::debug!("JWT verification failed: {}", e);
        ApiError::unauthorized("Invalid or expired token")
    })?;

    let claims = token_data.claims;

    // Check if token is expired (extra safety check)
    if claims.is_expired() {
        return Err(ApiError::unauthorized("Token has expired"));
    }

    Ok(claims)
}

/// Extract token from Authorization header
pub fn extract_token(auth_header: &str) -> ApiResult<&str> {
    // Expected format: "Bearer <token>"
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::unauthorized(
            "Invalid authorization header format",
        ));
    }

    let token = auth_header.trim_start_matches("Bearer ");
    if token.is_empty() {
        return Err(ApiError::unauthorized("Token is empty"));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let role = UserRole::Admin;

        let claims = Claims::new(user_id, email.clone(), role.clone());

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_extract_token() {
        let result = extract_token("Bearer abc123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc123");

        let result = extract_token("Basic abc123");
        assert!(result.is_err());

        let result = extract_token("Bearer ");
        assert!(result.is_err());
    }
}
