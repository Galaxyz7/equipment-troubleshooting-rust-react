use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("🔄 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("📦 Running node-graph refactor migration...");

    let migration_sql = include_str!("../../migrations/006_node_graph_refactor.sql");

    sqlx::raw_sql(migration_sql)
        .execute(&pool)
        .await?;

    println!("✅ Migration completed successfully!");
    println!("📊 Checking results...");

    // Check how many nodes were created
    let node_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nodes")
        .fetch_one(&pool)
        .await?;
    println!("  - Nodes created: {}", node_count.0);

    // Check how many connections were created
    let connection_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM connections")
        .fetch_one(&pool)
        .await?;
    println!("  - Connections created: {}", connection_count.0);

    Ok(())
}
