// Re-export modules
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

use sqlx::PgPool;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
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
