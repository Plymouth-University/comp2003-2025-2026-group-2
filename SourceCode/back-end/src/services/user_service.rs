use crate::db;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;

pub struct UserService;

impl UserService {
    pub async fn get_user_by_id(
        db_pool: &SqlitePool,
        user_id: &str,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        db::get_user_by_id(db_pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((StatusCode::NOT_FOUND, json!({ "error": "User not found" })))
    }

    pub async fn update_profile(
        db_pool: &SqlitePool,
        user_id: &str,
        first_name: String,
        last_name: String,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        db::update_user_profile(db_pool, user_id, first_name, last_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update profile: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to update profile" }),
                )
            })
    }

    pub async fn get_company_members(
        db_pool: &SqlitePool,
        company_id: &str,
    ) -> Result<Vec<db::UserRecord>, (StatusCode, serde_json::Value)> {
        db::get_users_by_company_id(db_pool, company_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching company members: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })
    }

    pub async fn get_user_company_id(
        db_pool: &SqlitePool,
        user_id: &str,
    ) -> Result<String, (StatusCode, serde_json::Value)> {
        db::get_user_company_id(db_pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching user company ID: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::FORBIDDEN,
                json!({ "error": "User is not associated with a company" }),
            ))
    }
}
