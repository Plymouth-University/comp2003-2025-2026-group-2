pub mod auth;
pub mod db;
pub mod handlers;
pub mod middleware;
pub mod rate_limit;

use sqlx::SqlitePool;
use rate_limit::RateLimitState;

#[derive(Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub rate_limit: RateLimitState,
}
