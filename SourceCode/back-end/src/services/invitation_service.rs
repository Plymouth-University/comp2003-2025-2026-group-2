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
}
