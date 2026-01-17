use crate::db;
use axum::http::HeaderMap;
use sqlx::PgPool;

pub struct AuditLogger;

impl AuditLogger {
    pub async fn log(
        db: &PgPool,
        event_type: &str,
        user_id: Option<String>,
        email: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<String>,
        success: bool,
    ) {
        let _ = db::log_security_event(
            db,
            event_type.to_string(),
            user_id,
            email,
            ip_address,
            user_agent,
            details,
            success,
        )
        .await;
    }

    pub async fn log_registration(
        db: &PgPool,
        user_id: String,
        email: String,
        company_name: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        Self::log(
            db,
            "registration",
            Some(user_id),
            Some(email),
            ip_address,
            user_agent,
            Some(format!("Company admin registered: {company_name}")),
            true,
        )
        .await;
    }

    pub async fn log_login_success(
        db: &PgPool,
        user_id: String,
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        Self::log(
            db,
            "login_success",
            Some(user_id),
            Some(email),
            ip_address,
            user_agent,
            None,
            true,
        )
        .await;
    }

    pub async fn log_login_failed(
        db: &PgPool,
        user_id: Option<String>,
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        reason: &str,
    ) {
        Self::log(
            db,
            "login_failed",
            user_id,
            Some(email),
            ip_address,
            user_agent,
            Some(reason.to_string()),
            false,
        )
        .await;
    }

    pub async fn log_invitation_sent(
        db: &PgPool,
        admin_id: String,
        admin_email: String,
        recipient_email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        Self::log(
            db,
            "invitation_sent",
            Some(admin_id),
            Some(recipient_email),
            ip_address,
            user_agent,
            Some(format!("Invitation sent by {admin_email}")),
            true,
        )
        .await;
    }

    pub async fn log_invitation_accepted(
        db: &PgPool,
        user_id: String,
        email: String,
        company_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        Self::log(
            db,
            "invitation_accepted",
            Some(user_id),
            Some(email),
            ip_address,
            user_agent,
            Some(format!("Member joined company {company_id}")),
            true,
        )
        .await;
    }

    pub async fn log_profile_updated(db: &PgPool, user_id: String, email: String) {
        Self::log(
            db,
            "profile_updated",
            Some(user_id),
            Some(email),
            None,
            None,
            None,
            true,
        )
        .await;
    }

    pub async fn log_admin_action(db: &PgPool, admin_user_id: String, action_description: String) {
        Self::log(
            db,
            "admin_action",
            Some(admin_user_id),
            None,
            None,
            None,
            Some(action_description),
            true,
        )
        .await;
    }

    pub async fn log_password_reset_requested(
        db: &PgPool,
        user_id: Option<String>,
        email: String,
        reason: Option<&str>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        let is_success = user_id.is_some();
        Self::log(
            db,
            "password_reset_requested",
            user_id,
            Some(email),
            ip_address,
            user_agent,
            reason.map(std::string::ToString::to_string),
            is_success,
        )
        .await;
    }

    pub async fn log_password_reset_completed(db: &PgPool, user_id: String) {
        Self::log(
            db,
            "password_reset_completed",
            Some(user_id),
            None,
            None,
            None,
            None,
            true,
        )
        .await;
    }
}

#[must_use] 
pub fn extract_ip_from_headers_and_addr(
    headers: &HeaderMap,
    addr: &std::net::SocketAddr,
) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map_or_else(|| addr.ip().to_string(), |s| s.trim().to_string())
}

pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(std::string::ToString::to_string)
}
