//! Test configuration and setup utilities

use sqlx::PgPool;

/// Creates a test `PostgreSQL` connection pool.
///
/// # Panics
/// Panics if the database connection fails.
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:adminpassword@localhost:5432/logsmartdb".to_string());

    // Create connection pool with test-specific settings
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

/// Test environment setup
pub fn setup_test_environment() {
    // Set environment variables for testing
    // SAFETY: This is safe because tests are run sequentially in this context or environment is set before threads spawn
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var(
            "TEST_DATABASE_URL",
            "postgres://admin:adminpassword@localhost:5432/logsmartdb",
        );
    }
}
