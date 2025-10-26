// Re-export modules
pub mod error;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod routes;
pub mod utils;

use sqlx::PgPool;
use crate::utils::cache::Cache;
use serde_json::Value as JsonValue;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    /// Cache for questions list (5 minute TTL)
    pub questions_cache: Cache<String, JsonValue>,
    /// Cache for issue trees (10 minute TTL)
    pub issue_tree_cache: Cache<String, JsonValue>,
    /// Cache for issue graphs (10 minute TTL)
    pub issue_graph_cache: Cache<String, JsonValue>,
}

impl AppState {
    /// Create a new AppState with initialized caches
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            // Cache questions for 5 minutes, max 10 entries
            questions_cache: Cache::new(300, 10),
            // Cache issue trees for 10 minutes, max 50 entries
            issue_tree_cache: Cache::new(600, 50),
            // Cache issue graphs for 10 minutes, max 50 entries
            issue_graph_cache: Cache::new(600, 50),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generate_typescript_types() {
        // This test triggers TypeScript generation for all types with #[derive(TS)]
        // Run: cargo test
        println!("TypeScript types generated in apps/web/src/types/");
    }
}
