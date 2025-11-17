pub mod auth;
pub mod db;
pub mod handlers;
pub mod middleware;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
}
