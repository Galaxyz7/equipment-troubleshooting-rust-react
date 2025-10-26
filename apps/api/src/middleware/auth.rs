use crate::error::{ApiError, ApiResult};
use crate::models::UserRole;
use crate::utils::jwt::{extract_token, verify_token, Claims};
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

/// Extension type to store authenticated user claims in request
#[derive(Clone, Debug)]
pub struct AuthUser(pub Claims);

/// Middleware to verify JWT token and extract user claims
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?;

    // Extract and verify token
    let token = extract_token(auth_header)?;
    let claims = verify_token(token)?;

    // Add claims to request extensions
    request.extensions_mut().insert(AuthUser(claims));

    // Continue to next handler
    Ok(next.run(request).await)
}

/// Middleware to require ADMIN role
pub async fn require_admin(
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    // First run auth middleware
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?;

    let token = extract_token(auth_header)?;
    let claims = verify_token(token)?;

    // Check if user is ADMIN
    if !matches!(claims.role, UserRole::Admin) {
        return Err(ApiError::forbidden(
            "This action requires administrator privileges",
        ));
    }

    // Add claims to request extensions
    request.extensions_mut().insert(AuthUser(claims));

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_clone() {
        use uuid::Uuid;

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::Admin,
            iat: 0,
            exp: 9999999999,
        };

        let auth_user = AuthUser(claims);
        let cloned = auth_user.clone();

        assert_eq!(auth_user.0.email, cloned.0.email);
    }
}
