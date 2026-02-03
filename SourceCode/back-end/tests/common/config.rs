//! Test configuration and setup utilities

use crate::{AppState, RateLimitState};
use mongodb::Client as MongoClient;
use sqlx::PgPool;
use std::sync::Arc;

/// Creates a test application state with mock components
pub async fn create_test_app_state() -> AppState {
    let postgres_pool = create_test_pool().await;
    let mongodb_client = create_test_mongodb_client().await;

    // Create mock components
    let rate_limit = RateLimitState::new();
    let metrics = crate::metrics::Metrics::new();

    // Mock WebAuthn, OAuth components (will be implemented in mocks)
    let webauthn = Arc::new(create_mock_webauthn());
    // google_oauth is Option, so we can pass None or Some(mock)
    let google_oauth = None; // Use None for now as mocking GoogleOAuthClient is hard (it's not a trait)
    let oauth_state_store = Arc::new(create_mock_oauth_state_store());

    let user_cache = moka::future::Cache::builder()
        .max_capacity(1_000)
        .time_to_live(std::time::Duration::from_secs(60))
        .build();

    AppState {
        postgres: postgres_pool,
        mongodb: mongodb_client,
        rate_limit,
        metrics,
        webauthn,
        google_oauth,
        oauth_state_store,
        user_cache,
    }
}

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

/// Creates a test `MongoDB` client.
///
/// # Panics
/// Panics if the `MongoDB` connection fails.
pub async fn create_test_mongodb_client() -> MongoClient {
    let mongodb_url = std::env::var("TEST_MONGODB_URL")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    MongoClient::with_uri_str(&mongodb_url)
        .await
        .expect("Failed to create test MongoDB client")
}

/// Mock `WebAuthn` configuration
#[must_use]
pub fn create_mock_webauthn() -> webauthn_rs::Webauthn {
    // This will be implemented properly in the mocks module
    // For now, return a placeholder
    unimplemented!("Mock WebAuthn implementation needed")
}

/// Mock Google OAuth configuration  
#[must_use]
pub fn create_mock_google_oauth() -> crate::services::GoogleOAuthClient {
    // This will be implemented properly in the mocks module
    unimplemented!("Mock Google OAuth implementation needed")
}

/// Mock OAuth state store
#[must_use]
pub fn create_mock_oauth_state_store() -> crate::handlers::OAuthStateStore {
    // This will be implemented properly in the mocks module
    unimplemented!("Mock OAuth state store implementation needed")
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
        std::env::set_var("TEST_MONGODB_URL", "mongodb://localhost:27017");
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_only");
    }
}

/// Cleanup test environment
pub fn cleanup_test_environment() {
    // Clean up environment variables
    unsafe {
        std::env::remove_var("RUST_LOG");
        std::env::remove_var("TEST_DATABASE_URL");
        std::env::remove_var("TEST_MONGODB_URL");
        std::env::remove_var("JWT_SECRET");
    }
}
