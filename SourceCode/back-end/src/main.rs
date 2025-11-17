use back_end::{db, handlers, rate_limit, AppState};
use anyhow::Context;
use axum::{middleware, Router};
use axum::routing::{get, post};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let connect_options = SqliteConnectOptions::new()
        .filename("auth.db")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .pragma("cache_size", "2000")
        .pragma("temp_store", "memory")
        .pragma("mmap_size", "268435456")
        .pragma("foreign_keys", "ON")
        .pragma("busy_timeout", "30000");

    let auth_db_sqlite_pool = SqlitePoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .idle_timeout(Some(std::time::Duration::from_secs(600)))
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_with(connect_options)
        .await
        .with_context(|| "Failed to create auth_db_sqlite_pool")
        .expect("Cannot create authentication db");

    db::init_db(&auth_db_sqlite_pool)
        .await
        .expect("Failed to initialize database");

    let rate_limit_state = rate_limit::RateLimitState::new();

    let state = AppState {
        sqlite: auth_db_sqlite_pool,
        rate_limit: rate_limit_state.clone(),
    };

    let app = Router::new()
        .route("/auth/register", post(handlers::register_company_admin))
        .route("/auth/login", post(handlers::login))
        .route("/auth/me", get(handlers::get_current_user))
        .route("/auth/invitations/send", post(handlers::invite_user))
        .route("/auth/invitations/accept", post(handlers::accept_invitation))
        .layer(middleware::from_fn_with_state(
            rate_limit_state,
            rate_limit::rate_limit_middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
