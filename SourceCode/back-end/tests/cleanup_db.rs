/// Run this before running http_api_tests to clean the database
/// Usage: cargo test --test cleanup_db
use sqlx::PgPool;

#[tokio::test]
async fn cleanup_test_database() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:adminpassword@localhost:5432/logsmartdb".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test db");

    println!("Cleaning up test database...");

    let _ = sqlx::query("DELETE FROM security_logs")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM passkey_sessions")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM passkeys").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM invitations").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM users").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM companies").execute(&pool).await;

    println!("Test database cleaned successfully!");
}
