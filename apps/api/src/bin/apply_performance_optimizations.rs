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

    // Apply performance optimization indexes
    println!("Creating performance optimization indexes...");

    // 1. Start session optimization
    println!("  ✓ Creating idx_nodes_semantic_active...");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_nodes_semantic_active
         ON nodes(semantic_id, is_active)
         WHERE is_active = true"
    )
    .execute(&pool)
    .await?;

    // 2. Connection queries optimization
    println!("  ✓ Creating idx_connections_from_active_order...");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_connections_from_active_order
         ON connections(from_node_id, is_active, order_index)
         WHERE is_active = true"
    )
    .execute(&pool)
    .await?;

    // 3. Connection JOIN optimization
    println!("  ✓ Creating idx_connections_from_with_target...");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_connections_from_with_target
         ON connections(from_node_id, is_active, to_node_id, order_index)
         WHERE is_active = true"
    )
    .execute(&pool)
    .await?;

    // 4. Covering index for node lookups
    println!("  ✓ Creating idx_nodes_active_complete...");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_nodes_active_complete
         ON nodes(is_active, id)
         INCLUDE (category, node_type, text, semantic_id, display_category)
         WHERE is_active = true"
    )
    .execute(&pool)
    .await?;

    // 5. Category filtering
    println!("  ✓ Creating idx_nodes_category_active...");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_nodes_category_active
         ON nodes(category, is_active)
         WHERE is_active = true"
    )
    .execute(&pool)
    .await?;

    println!("✓ All performance indexes created");

    // Mark migration as applied
    println!("Marking migration as applied...");
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES (10, 'performance optimizations', true, decode('', 'hex'), 0)
         ON CONFLICT (version) DO NOTHING"
    )
    .execute(&pool)
    .await?;
    println!("✓ Migration marked as applied");

    println!("\n✅ Performance optimization migration completed successfully!");

    Ok(())
}
