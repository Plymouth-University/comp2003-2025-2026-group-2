use crate::{
    auth::{generate_uuid6_token, hash_password, validate_password_policy, verify_password},
    db,
    jwt_manager::JwtManager,
    utils::AuditLogger,
};
use axum::http::StatusCode;
use chrono::Duration;
use serde_json::json;
use sqlx::SqlitePool;

pub struct AuthService;

impl AuthService {
    pub async fn register_admin(
        db_pool: &SqlitePool,
        email: &str,
        first_name: &str,
        last_name: &str,
        password: &str,
        company_name: &str,
        company_address: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String, String), (StatusCode, serde_json::Value)> {
        let mut tx = db_pool.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Database transaction error" }),
            )
        })?;

        let user_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let role_str = db::UserRole::Admin.to_string();
        let password_hash = hash_password(password).map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to process password" }),
            )
        })?;

        sqlx::query(
            r"
            INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
            VALUES (?, ?, ?, ?, ?, NULL, ?, ?)
            ",
        )
        .bind(&user_id)
        .bind(email)
        .bind(first_name)
        .bind(last_name)
        .bind(&password_hash)
        .bind(&role_str)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user in transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to create user" }),
            )
        })?;

        let company_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r"
            INSERT INTO companies (id, name, address, created_at)
            VALUES (?, ?, ?, ?)
            ",
        )
        .bind(&company_id)
        .bind(company_name)
        .bind(company_address)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create company in transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to create company" }),
            )
        })?;

        sqlx::query("UPDATE users SET company_id = ? WHERE id = ?")
            .bind(&company_id)
            .bind(&user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to link user to company: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to link user to company" }),
                )
            })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit registration transaction: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to commit transaction" }),
            )
        })?;

        let jwt_config = JwtManager::get_config();
        let token = jwt_config
            .generate_token(user_id.clone(), 24)
            .map_err(|e| {
                tracing::error!("Failed to generate JWT token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to generate token" }),
                )
            })?;

        AuditLogger::log_registration(
            db_pool,
            user_id.clone(),
            email.to_string(),
            company_name.to_string(),
            ip_address,
            user_agent,
        )
        .await;

        Ok((token, user_id, role_str))
    }

    pub async fn verify_credentials(
        db_pool: &SqlitePool,
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

        if user.is_none() {
            AuditLogger::log_login_failed(
                db_pool,
                None,
                email.to_string(),
                ip_address,
                user_agent,
                "User not found",
            )
            .await;
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Invalid email or password" }),
            ));
        }

        let user = user.unwrap();
        let password_valid = verify_password(password, &user.password_hash).map_err(|e| {
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

        let jwt_config = JwtManager::get_config();
        let token = jwt_config
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
    }

    pub async fn reset_password(
        db_pool: &SqlitePool,
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

    pub async fn request_password_reset(
        db_pool: &SqlitePool,
        email: &str,
    ) -> Result<String, (StatusCode, serde_json::Value)> {
        let user = db::get_user_by_email(db_pool, email).await.map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Database error" }),
            )
        })?;

        if let Some(user_record) = user {
            let reset_token = generate_uuid6_token();
            let expires_at = (chrono::Utc::now() + Duration::hours(24)).to_rfc3339();

            let _ = db::create_password_reset_token(
                db_pool,
                user_record.id.clone(),
                reset_token.clone(),
                expires_at,
            )
            .await;

            AuditLogger::log_password_reset_requested(
                db_pool,
                Some(user_record.id),
                email.to_string(),
                None,
            )
            .await;

            Ok(reset_token)
        } else {
            AuditLogger::log_password_reset_requested(
                db_pool,
                None,
                email.to_string(),
                Some("User not found"),
            )
            .await;
            Ok(String::new())
        }
    }
}
