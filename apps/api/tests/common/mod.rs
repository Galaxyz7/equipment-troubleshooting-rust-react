use equipment_troubleshooting::models::UserRole;
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

/// Test database connection pool
pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/equipment_troubleshooting_test".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Clean up test data from database
pub async fn cleanup_test_db(pool: &PgPool) {
    // Clean up in reverse order of foreign keys
    let _ = sqlx::query("DELETE FROM connections").execute(pool).await;
    let _ = sqlx::query("DELETE FROM nodes").execute(pool).await;
    let _ = sqlx::query("DELETE FROM users WHERE email LIKE '%@test.com'").execute(pool).await;
}

/// Create a test user and return ID
pub async fn create_test_user(pool: &PgPool, email: &str, role: UserRole) -> Uuid {
    use argon2::{
        password_hash::{PasswordHasher, SaltString},
        Argon2
    };
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(b"testpassword123", &salt)
        .unwrap()
        .to_string();

    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, role, is_active) VALUES ($1, $2, $3, $4, true)"
    )
    .bind(user_id)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Generate JWT token for test user
pub fn generate_test_token(user_id: Uuid, email: &str, role: UserRole) -> String {
    equipment_troubleshooting::utils::jwt::generate_token(user_id, email.to_string(), role)
        .expect("Failed to generate test token")
}

/// Create a test issue (category) and return root node ID
pub async fn create_test_issue(pool: &PgPool, category: &str, name: &str) -> Uuid {
    use equipment_troubleshooting::models::NodeType;

    let root_node_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO nodes (id, category, node_type, text, semantic_id, is_active, position_x, position_y)
         VALUES ($1, $2, $3, $4, $5, true, 0, 0)"
    )
    .bind(root_node_id)
    .bind(category)
    .bind(NodeType::Question)
    .bind(format!("{} - Root Question", name))
    .bind("root")
    .execute(pool)
    .await
    .expect("Failed to create test issue");

    root_node_id
}

/// Create a test connection between nodes
pub async fn create_test_connection(
    pool: &PgPool,
    from_node_id: Uuid,
    to_node_id: Uuid,
    label: &str,
) -> Uuid {
    let connection_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO connections (id, from_node_id, to_node_id, label, order_index, is_active)
         VALUES ($1, $2, $3, $4, 0, true)"
    )
    .bind(connection_id)
    .bind(from_node_id)
    .bind(to_node_id)
    .bind(label)
    .execute(pool)
    .await
    .expect("Failed to create test connection");

    connection_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let pool = setup_test_db().await;
        assert!(pool.acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_create_test_user() {
        let pool = setup_test_db().await;
        let user_id = create_test_user(&pool, "test@test.com", UserRole::Admin).await;
        assert!(!user_id.is_nil());
        cleanup_test_db(&pool).await;
    }
}
