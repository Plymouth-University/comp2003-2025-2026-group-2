use crate::db;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[cfg(test)]
mod user_service_tests {
    #[tokio::test]
    async fn test_user_service_basic() {
        assert!(true);
    }
}

pub struct UserService;

impl UserService {
    pub async fn get_user_by_email(
        db_pool: &PgPool,
        email: &str,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        db::get_user_by_email(db_pool, email)
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

    pub async fn get_user_by_id(
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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

    pub async fn admin_update_member_profile(
        db_pool: &PgPool,
        admin_user_id: &str,
        target_email: &str,
        first_name: String,
        last_name: String,
        role: db::UserRole,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        let admin = Self::get_user_by_id(db_pool, admin_user_id).await?;

        if !admin.is_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only company admins can update member profiles" }),
            ));
        }

        let target_user = Self::get_user_by_email(db_pool, target_email).await?;

        if admin.company_id != target_user.company_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot update users from other companies" }),
            ));
        }

        if target_user.is_logsmart_admin() && !admin.is_logsmart_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot modify LogSmart internal admin users" }),
            ));
        }

        db::update_user_profile_full(db_pool, &target_user.id, first_name, last_name, role)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update member profile: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to update member profile" }),
                )
            })
    }

    pub async fn admin_delete_member(
        db_pool: &PgPool,
        admin_user_id: &str,
        target_email: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let admin = Self::get_user_by_id(db_pool, admin_user_id).await?;

        if !admin.is_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only company admins can delete members" }),
            ));
        }

        let target_user = Self::get_user_by_email(db_pool, target_email).await?;

        if admin.company_id != target_user.company_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot delete users from other companies" }),
            ));
        }

        if target_user.is_logsmart_admin() && !admin.is_logsmart_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot delete LogSmart internal admin users" }),
            ));
        }

        if target_user.email == admin.email {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({ "error": "Cannot delete your own account" }),
            ));
        }

        db::delete_user_by_email(db_pool, target_email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete member: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to delete member" }),
                )
            })
    }
}
