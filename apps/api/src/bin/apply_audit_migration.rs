use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database");

    // Drop existing audit_logs table if it exists
    println!("Dropping existing audit_logs table if it exists...");
    sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
        .execute(&pool)
        .await?;
    println!("✓ Old table dropped");

    // Create audit_logs table
    println!("Creating audit_logs table...");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            action VARCHAR(100) NOT NULL,
            resource_type VARCHAR(50) NOT NULL,
            resource_id VARCHAR(255),
            details JSONB,
            ip_address VARCHAR(45),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"
    )
    .execute(&pool)
    .await?;
    println!("✓ Table created");

    // Create indexes
    println!("Creating indexes...");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_type ON audit_logs(resource_type)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_id ON audit_logs(resource_id)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_user_time ON audit_logs(user_id, created_at DESC)")
        .execute(&pool)
        .await?;
    println!("✓ Indexes created");

    // Mark migration as applied
    println!("Marking migration as applied...");
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES (9, 'create audit logs', true, decode('', 'hex'), 0)
         ON CONFLICT (version) DO NOTHING"
    )
    .execute(&pool)
    .await?;
    println!("✓ Migration marked as applied");

    println!("\n✅ Audit logs migration completed successfully!");

    Ok(())
}
