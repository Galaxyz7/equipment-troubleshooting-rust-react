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

    // Apply display_category column
    println!("Adding display_category column...");
    sqlx::query("ALTER TABLE nodes ADD COLUMN IF NOT EXISTS display_category VARCHAR(255)")
        .execute(&pool)
        .await?;
    println!("✓ Column added");

    // Create index
    println!("Creating index...");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_display_category ON nodes(display_category)")
        .execute(&pool)
        .await?;
    println!("✓ Index created");

    // Update existing nodes
    println!("Updating existing nodes...");
    let result = sqlx::query("UPDATE nodes SET display_category = 'General' WHERE display_category IS NULL")
        .execute(&pool)
        .await?;
    println!("✓ Updated {} rows", result.rows_affected());

    // Mark migrations as applied
    println!("Marking migrations as applied...");
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES
           (6, 'node graph refactor', true, decode('', 'hex'), 0),
           (7, 'add issue category', true, decode('', 'hex'), 0)
         ON CONFLICT (version) DO NOTHING"
    )
    .execute(&pool)
    .await?;
    println!("✓ Migrations marked as applied");

    println!("\n✅ Migration completed successfully!");

    Ok(())
}
