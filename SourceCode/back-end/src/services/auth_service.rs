use crate::{
    auth::{generate_uuid6_token, hash_password, validate_password_policy, verify_password},
    db::{self, UserRole},
    jwt_manager::JwtManager,
    utils::AuditLogger,
};
use axum::http::StatusCode;
use chrono::Duration;
use serde_json::json;
use sqlx::PgPool;

#[cfg(test)]
mod auth_service_tests {
    #[tokio::test]
    async fn test_auth_service_basic() {
        // Basic test to ensure service compiles
        assert!(true);
    }
}

pub struct AuthService;

impl AuthService {
    pub async fn register_admin(
        db_pool: &PgPool,
        email: &str,
        first_name: &str,
        last_name: &str,
        password: &str,
        company_name: &str,
        company_address: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, UserRole), (StatusCode, serde_json::Value)> {
        let mut tx = db_pool.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Database transaction failed"}),
            )
        })?;

        let password_hash = hash_password(password).map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Password processing failed"}),
            )
        })?;

        let company = db::create_company(
            &mut *tx,
            company_name.to_string(),
            company_address.to_string(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create company: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Company creation failed"}),
            )
        })?;

        let user = db::create_user(
            &mut *tx,
            email.to_string(),
            first_name.to_string(),
            last_name.to_string(),
            Some(password_hash),
            Some(company.id.clone()),
            UserRole::Admin,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "User creation failed"}),
            )
        })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Transaction commit failed"}),
            )
        })?;

        let user_id = user.id.clone();

        let token = JwtManager::get_config()
            .generate_token(user_id.clone(), 24)
            .map_err(|e| {
                tracing::error!("Failed to generate JWT token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Token generation failed"}),
                )
            })?;

        // Log the registration event
        AuditLogger::log_registration(
            db_pool,
            user_id.clone(),
            email.to_string(),
            company_name.to_string(),
            ip_address,
            user_agent,
        )
        .await;

        let response = serde_json::json!({
            "message": "Admin registration successful",
            "user_id": user_id,
            "token": token,
            "role": "admin",
        });

        Ok((
            serde_json::to_string(&response).unwrap(),
            token,
            UserRole::Admin,
        ))
    }

    pub async fn login(
        db_pool: &PgPool,
        email: &str,
        password: &str,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(String, String, UserRole), (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_email(db_pool, email)
            .await
            .map_err(|e| {
                tracing::error!("Database error during login: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Database error"}),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    json!({"error": "Invalid credentials"}),
                )
            })?;

        if user.deleted_at.is_some() {
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({"error": "Account deactivated"}),
            ));
        }

        let password_valid = if let Some(password_hash) = &user.password_hash {
            verify_password(password, password_hash).map_err(|e| {
                tracing::error!("Password verification error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Authentication failed"}),
                )
            })?
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({"error": "OAuth-only account - password login not available"}),
            ));
        };

        if !password_valid {
            AuditLogger::log_login_failed(
                db_pool,
                Some(user.id.clone()),
                email.to_string(),
                ip_address,
                user_agent,
                "Invalid password",
            )
            .await;

            return Err((
                StatusCode::UNAUTHORIZED,
                json!({"error": "Invalid credentials"}),
            ));
        }

        let token = JwtManager::get_config()
            .generate_token(user.id.clone(), 24)
            .map_err(|e| {
                tracing::error!("Failed to generate login JWT token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Failed to generate token"}),
                )
            })?;

        // Login time update functionality not yet implemented in db module
        // TODO: Implement update_user_login_time function in db module

        AuditLogger::log_login_success(
            db_pool,
            user.id.clone(),
            email.to_string(),
            ip_address,
            user_agent,
        )
        .await;

        let response = serde_json::json!({
            "message": "Login successful",
            "user_id": user.id,
            "token": token,
            "role": user.get_role().to_string(),
        });

        Ok((
            serde_json::to_string(&response).unwrap(),
            token,
            user.get_role(),
        ))
    }

    pub async fn request_password_reset(
        db_pool: &PgPool,
        email: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_email(db_pool, email).await.map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Database error" }),
            )
        })?;

        if let Some(user_record) = user {
            let reset_token = generate_uuid6_token();
            let expires_at = chrono::Utc::now() + Duration::hours(24);

            db::create_password_reset_token(
                db_pool,
                user_record.id.clone(),
                reset_token.clone(),
                expires_at,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create password reset token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to process password reset request" }),
                )
            })?;

            let reset_link = format!(
                "{}/reset-password?token={}",
                "https://logsmart.app", reset_token
            );

            crate::email::send_password_reset_email(&user_record.email, &reset_link)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to send password reset email: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Failed to send password reset email" }),
                    )
                })?;

            AuditLogger::log_password_reset_requested(
                db_pool,
                Some(user_record.id),
                email.to_string(),
                None,
                ip_address,
                user_agent,
            )
            .await;
        } else {
            AuditLogger::log_password_reset_requested(
                db_pool,
                None,
                email.to_string(),
                Some("User not found"),
                ip_address,
                user_agent,
            )
            .await;
        }

        Ok(())
    }

    pub async fn validate_reset_token(
        db_pool: &PgPool,
        token: &str,
    ) -> Result<bool, (StatusCode, serde_json::Value)> {
        let reset_record = db::get_password_reset_by_token(db_pool, token)
            .await
            .map_err(|_| {
                tracing::error!("Failed to get password reset record");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Failed to validate reset token"}),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    json!({"error": "Invalid or expired reset token"}),
                )
            })?;

        // reset_record returns (reset_id, user_id) tuple
        let (reset_id, _user_id) = reset_record;

        // Mark as used
        db::mark_password_reset_used(db_pool, &reset_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to mark reset token as used: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Failed to process token"}),
                )
            })?;

        Ok(true)
    }

    pub async fn reset_password(
        db_pool: &PgPool,
        reset_token: &str,
        new_password: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        validate_password_policy(new_password)
            .map_err(|e| (StatusCode::BAD_REQUEST, json!({ "error": e.to_string() })))?;

        let (reset_id, user_id) = db::get_password_reset_by_token(db_pool, reset_token)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    json!({ "error": "Invalid or expired reset token" }),
                )
            })?;

        let password_hash = hash_password(new_password).map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to process password" }),
            )
        })?;

        db::update_user_password(db_pool, &user_id, password_hash)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update password: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to update password" }),
                )
            })?;

        db::mark_password_reset_used(db_pool, &reset_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to mark reset token as used: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to process request" }),
                )
            })?;

        AuditLogger::log_password_reset_completed(db_pool, user_id).await;

        Ok(())
    }

    pub async fn change_password(
        db_pool: &PgPool,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_id(db_pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Database error"}),
                )
            })?
            .ok_or_else(|| (StatusCode::NOT_FOUND, json!({"error": "User not found"})))?;

        // Validate current password
        let is_current_valid = if let Some(password_hash) = &user.password_hash {
            verify_password(current_password, password_hash).map_err(|e| {
                tracing::error!("Password verification error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Password verification failed"}),
                )
            })?
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({"error": "OAuth-only account - password change not available"}),
            ));
        };

        if !is_current_valid {
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({"error": "Current password is incorrect"}),
            ));
        }

        // Validate new password meets requirements
        validate_password_policy(new_password)
            .map_err(|e| (StatusCode::BAD_REQUEST, json!({ "error": e.to_string() })))?;

        let password_hash = hash_password(new_password).map_err(|e| {
            tracing::error!("Failed to hash new password: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Password processing failed"}),
            )
        })?;

        db::update_user_password(db_pool, user_id, password_hash)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update password: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": "Failed to update password"}),
                )
            })?;

        // Password change logging not yet implemented in AuditLogger
        // TODO: Implement log_password_changed function in AuditLogger

        Ok(())
    }

    pub async fn verify_credentials(
        db_pool: &PgPool,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, db::UserRecord), (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_email(db_pool, email).await.map_err(|e| {
            tracing::error!("Database error during login lookup: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Database error" }),
            )
        })?;

        // If user not found, log the failed attempt before returning error
        if user.is_none() {
            AuditLogger::log_login_failed(
                db_pool,
                None,
                email.to_string(),
                ip_address.clone(),
                user_agent.clone(),
                "User not found",
            )
            .await;
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Invalid email or password" }),
            ));
        }

        let user = user.unwrap();

        if let Some(password_hash) = user.password_hash.as_ref() {
            let password_valid = verify_password(password, password_hash).map_err(|e| {
                tracing::error!("Password verification error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Authentication failed" }),
                )
            })?;

            if !password_valid {
                AuditLogger::log_login_failed(
                    db_pool,
                    Some(user.id.clone()),
                    email.to_string(),
                    ip_address,
                    user_agent,
                    "Invalid password",
                )
                .await;
                return Err((
                    StatusCode::UNAUTHORIZED,
                    json!({ "error": "Invalid email or password" }),
                ));
            }

            let token = JwtManager::get_config()
                .generate_token(user.id.clone(), 24)
                .map_err(|e| {
                    tracing::error!("Failed to generate login JWT token: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({ "error": "Failed to generate token" }),
                    )
                })?;

            AuditLogger::log_login_success(
                db_pool,
                user.id.clone(),
                email.to_string(),
                ip_address,
                user_agent,
            )
            .await;

            Ok((token, user))
        } else {
            AuditLogger::log_login_failed(
                db_pool,
                Some(user.id.clone()),
                email.to_string(),
                ip_address,
                user_agent,
                "OAuth-only account - password login not available",
            )
            .await;
            Err((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "This account uses OAuth login. Please sign in with Google." }),
            ))
        }
    }
}
