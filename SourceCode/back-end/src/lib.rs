pub mod api_docs;
pub mod auth;
pub mod db;
pub mod dto;
pub mod email;
pub mod handlers;
pub mod jwt_manager;
pub mod llm;
pub mod logs_db;
pub mod metrics;
pub mod middleware;
pub mod rate_limit;
pub mod security;
pub mod services;
pub mod utils;

use rate_limit::RateLimitState;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgPool,
    pub rate_limit: RateLimitState,
    pub metrics: metrics::Metrics,
    pub mongodb: mongodb::Client,
    pub webauthn: std::sync::Arc<webauthn_rs::Webauthn>,
}
