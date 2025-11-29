pub mod api_docs;
pub mod auth;
pub mod db;
pub mod dto;
pub mod email;
pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod rate_limit;
pub mod logs_db;

use rate_limit::RateLimitState;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub rate_limit: RateLimitState,
    pub metrics: metrics::Metrics,
    pub mongodb: mongodb::Client,
}
