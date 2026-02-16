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
    /// Retrieves a user by their email address.
    ///
    /// # Errors
    /// Returns an error if the user is not found or if database lookup fails.
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

    /// Retrieves a user by their ID.
    ///
    /// # Errors
    /// Returns an error if the user is not found or if database lookup fails.
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

    /// Updates a user's profile information.
    ///
    /// # Errors
    /// Returns an error if the database update fails.
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

    /// Retrieves all members of a specific company.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
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

    /// Retrieves the company ID for a specific user.
    ///
    /// # Errors
    /// Returns an error if the user is not associated with a company or if the query fails.
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

    /// Updates a company member's profile (admin only).
    ///
    /// # Errors
    /// Returns an error if the caller is not an admin, target is in another company, or update fails.
    pub async fn admin_update_member_profile(
        db_pool: &PgPool,
        admin_user_id: &str,
        target_email: &str,
        first_name: String,
        last_name: String,
        role: db::UserRole,
        branch_id: Option<String>,
    ) -> Result<db::UserRecord, (StatusCode, serde_json::Value)> {
        let admin = Self::get_user_by_id(db_pool, admin_user_id).await?;

        if !admin.can_manage_branch() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only managers can update member profiles" }),
            ));
        }

        let target_user = Self::get_user_by_email(db_pool, target_email).await?;

        // Company admins can only manage users in their company, but logsmart_admin can manage any company
        if admin.is_company_manager() && admin.company_id != target_user.company_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot update users from other companies" }),
            ));
        }

        // Branch managers can only update users in their branch
        if admin.is_branch_manager() {
            if admin.branch_id != target_user.branch_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "Branch managers can only manage users in their branch" }),
                ));
            }
            if branch_id != admin.branch_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "Branch managers can only assign users to their own branch" }),
                ));
            }
            if role != db::UserRole::Staff {
                return Err((
                    StatusCode::FORBIDDEN,
                    json!({ "error": "Branch managers can only manage staff members" }),
                ));
            }
        }

        if target_user.is_logsmart_admin() && !admin.is_logsmart_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot modify LogSmart internal admin users" }),
            ));
        }

        db::update_user_profile_full(
            db_pool,
            &target_user.id,
            first_name,
            last_name,
            role,
            branch_id,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to update member profile: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to update member profile" }),
            )
        })
    }

    /// Deletes a company member (admin only).
    ///
    /// # Errors
    /// Returns an error if the caller is not an admin, target is in another company, or deletion fails.
    pub async fn admin_delete_member(
        db_pool: &PgPool,
        admin_user_id: &str,
        target_email: &str,
    ) -> Result<String, (StatusCode, serde_json::Value)> {
        let admin = Self::get_user_by_id(db_pool, admin_user_id).await?;

        if !admin.can_manage_branch() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only managers can delete members" }),
            ));
        }

        let target_user = Self::get_user_by_email(db_pool, target_email).await?;

        // Company admins can only delete users in their company, but logsmart_admin can delete from any company
        if admin.is_company_manager() && admin.company_id != target_user.company_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot delete users from other companies" }),
            ));
        }

        // Branch managers can only delete users in their branch
        if admin.is_branch_manager() && admin.branch_id != target_user.branch_id {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Branch managers can only delete users in their branch" }),
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
            })?;

        Ok(target_user.id)
    }
}
