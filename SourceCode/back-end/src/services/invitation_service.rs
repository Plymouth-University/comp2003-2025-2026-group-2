use crate::{auth::generate_uuid6_token, db, email, utils::AuditLogger};
use axum::http::StatusCode;
use chrono::Duration;
use serde_json::json;
use sqlx::PgPool;

pub struct InvitationService;

impl InvitationService {
    pub async fn send_invitation(
        db_pool: &PgPool,
        admin_id: String,
        admin_email: String,
        recipient_email: String,
        company_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, chrono::DateTime<chrono::Utc>), (StatusCode, serde_json::Value)> {
        if let Some(_existing_user) = db::get_user_by_email(db_pool, &recipient_email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check existing user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
        {
            return Err((
                StatusCode::CONFLICT,
                json!({ "error": "User already registered" }),
            ));
        }

        let token = generate_uuid6_token();
        let expires_at = chrono::Utc::now() + Duration::days(7);

        let invitation = db::create_invitation(
            db_pool,
            company_id,
            recipient_email.clone(),
            token,
            expires_at,
        )
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                tracing::warn!(
                    "Duplicate invitation attempt for email: {}",
                    recipient_email
                );
                (
                    StatusCode::CONFLICT,
                    json!({ "error": "User already invited" }),
                )
            } else {
                tracing::error!("Failed to create invitation: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to create invitation" }),
                )
            }
        })?;

        let invite_link = format!(
            "{}/accept-invitation?token={}",
            "https://logsmart.app", invitation.token
        );

        email::send_invitation_email(&recipient_email, &invite_link, "Your Company Name")
            .await
            .map_err(|e| {
                tracing::error!("Failed to send invitation email: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to send invitation email" }),
                )
            })?;

        AuditLogger::log_invitation_sent(
            db_pool,
            admin_id,
            admin_email,
            recipient_email,
            ip_address,
            user_agent,
        )
        .await;

        Ok((invitation.id, invitation.expires_at))
    }

    pub async fn accept_invitation(
        db_pool: &PgPool,
        token: &str,
    ) -> Result<
        (db::Invitation, chrono::DateTime<chrono::FixedOffset>),
        (StatusCode, serde_json::Value),
    > {
        let invitation = db::get_invitation_by_token(db_pool, token)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching invitation by token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Invalid or expired invitation" }),
            ))?;

        let now = chrono::Utc::now();
        let expires_at = invitation.expires_at.fixed_offset();

        if now > expires_at {
            return Err((
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Invitation has expired" }),
            ));
        }

        Ok((invitation, expires_at))
    }

    pub async fn get_invitation_details(
        db_pool: &PgPool,
        token: &str,
    ) -> Result<(String, chrono::DateTime<chrono::Utc>), (StatusCode, serde_json::Value)> {
        let invitation = db::get_invitation_by_token(db_pool, token)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching invitation by token: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Invitation not found" }),
            ))?;

        let company = db::get_company_by_id(db_pool, &invitation.company_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching company name: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Company not found" }),
            ))?;

        Ok((company.name, invitation.expires_at))
    }

    pub async fn mark_invitation_accepted(
        db_pool: &PgPool,
        invitation_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let accept_time = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r"
            UPDATE invitations
            SET accepted_at = ?
            WHERE id = ?
            ",
        )
        .bind(&accept_time)
        .bind(invitation_id)
        .execute(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark invitation as accepted: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Failed to accept invitation" }),
            )
        })?;

        Ok(())
    }

    pub async fn get_pending_invitations(
        db_pool: &PgPool,
        company_id: &str,
    ) -> Result<Vec<db::Invitation>, (StatusCode, serde_json::Value)> {
        db::get_pending_invitations_by_company_id(db_pool, company_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching pending invitations: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })
    }

    pub async fn cancel_invitation(
        db_pool: &PgPool,
        admin_user_id: &str,
        invitation_id: &str,
    ) -> Result<(), (StatusCode, serde_json::Value)> {
        let admin = db::get_user_by_id(db_pool, admin_user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching admin user: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Admin user not found" }),
            ))?;

        if !admin.is_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Only company admins can cancel invitations" }),
            ));
        }

        let invitation = db::get_invitation_by_id(db_pool, invitation_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching invitation: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Database error" }),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                json!({ "error": "Invitation not found" }),
            ))?;

        if admin.company_id != Some(invitation.company_id.clone()) {
            return Err((
                StatusCode::FORBIDDEN,
                json!({ "error": "Cannot cancel invitations from other companies" }),
            ));
        }

        if invitation.accepted_at.is_some() {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({ "error": "Cannot cancel an accepted invitation" }),
            ));
        }

        if invitation.cancelled_at.is_some() {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({ "error": "Invitation already cancelled" }),
            ));
        }

        let cancelled_invitation = db::cancel_invitation(db_pool, invitation_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to cancel invitation: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "Failed to cancel invitation" }),
                )
            })?;

        email::send_invitation_cancelled_email(&cancelled_invitation.email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to send cancellation email: {:?}", e);
            })
            .ok();

        AuditLogger::log_admin_action(
            db_pool,
            admin_user_id.to_string(),
            format!("Cancelled invitation for: {}", cancelled_invitation.email),
        )
        .await;

        Ok(())
    }
}
