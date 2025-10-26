mod common;

use equipment_troubleshooting::models::UserRole;
use equipment_troubleshooting::utils::jwt::{generate_token, verify_token, extract_token};
use uuid::Uuid;

#[tokio::test]
async fn test_generate_and_verify_token() {
    // Set JWT_SECRET for testing
    std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let role = UserRole::Admin;

    // Generate token
    let token = generate_token(user_id, email.clone(), role.clone())
        .expect("Failed to generate token");

    assert!(!token.is_empty());

    // Verify token
    let claims = verify_token(&token).expect("Failed to verify token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert!(matches!(claims.role, UserRole::Admin));
    assert!(!claims.is_expired());
}

#[tokio::test]
async fn test_extract_token_from_header() {
    let auth_header = "Bearer abc123xyz";
    let token = extract_token(auth_header).expect("Failed to extract token");
    assert_eq!(token, "abc123xyz");
}

#[tokio::test]
async fn test_extract_token_invalid_format() {
    let auth_header = "Basic abc123";
    let result = extract_token(auth_header);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_extract_token_empty() {
    let auth_header = "Bearer ";
    let result = extract_token(auth_header);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_invalid_token() {
    std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_purposes");

    let result = verify_token("invalid.token.here");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_and_cleanup_test_user() {
    let pool = common::setup_test_db().await;

    let user_id = common::create_test_user(&pool, "integration@test.com", UserRole::Viewer).await;

    // Verify user was created
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to check user existence");

    assert!(user_exists);

    // Cleanup
    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_password_hashing() {
    use argon2::{
        password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2
    };
    use rand::rngs::OsRng;

    let password = b"my_secure_password";
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password, &salt)
        .expect("Failed to hash password")
        .to_string();

    // Verify the password
    let parsed_hash = PasswordHash::new(&hash).expect("Failed to parse hash");
    let result = Argon2::default().verify_password(password, &parsed_hash);

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_user_roles() {
    // Test that user roles are correctly typed
    let admin = UserRole::Admin;
    let viewer = UserRole::Viewer;
    let tech = UserRole::Tech;

    assert!(matches!(admin, UserRole::Admin));
    assert!(matches!(viewer, UserRole::Viewer));
    assert!(matches!(tech, UserRole::Tech));
}
