pub mod auth;
pub mod db;
pub mod dto;
pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod rate_limit;

use sqlx::SqlitePool;
use rate_limit::RateLimitState;

#[derive(Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub rate_limit: RateLimitState,
    pub metrics: metrics::Metrics,
}
