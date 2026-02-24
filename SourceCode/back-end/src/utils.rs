use crate::db;
use axum::http::HeaderMap;
use sqlx::PgPool;

#[macro_export]
macro_rules! try_db {
    ($expr:expr, $context:literal) => {
        $expr.await.map_err(|e| {
            tracing::error!(error = ?e, context = $context, "Database error");
            $crate::utils::svc_err_internal($context)
        })
    };
}

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
        if let Err(e) = db::log_security_event(
            db,
            event_type.to_string(),
            user_id,
            email,
            ip_address,
            user_agent,
            details,
            success,
        )
        .await
        {
            tracing::error!("Failed to log security event: {:?}", e);
        }
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

    pub async fn log_password_changed(db: &PgPool, user_id: String, email: String) {
        Self::log(
            db,
            "password_changed",
            Some(user_id),
            Some(email),
            None,
            None,
            Some("User changed their password".to_string()),
            true,
        )
        .await;
    }

    pub async fn log_oauth_login(
        db: &PgPool,
        user_id: String,
        email: String,
        provider: String,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) {
        Self::log(
            db,
            "oauth_login",
            Some(user_id),
            Some(email),
            ip_address,
            user_agent,
            Some(format!("OAuth login via {provider}")),
            success,
        )
        .await;
    }

    pub async fn log_oauth_account_linked(
        db: &PgPool,
        user_id: String,
        email: String,
        provider: String,
    ) {
        Self::log(
            db,
            "oauth_account_linked",
            Some(user_id),
            Some(email),
            None,
            None,
            Some(format!("Linked {provider} account")),
            true,
        )
        .await;
    }

    pub async fn log_oauth_account_unlinked(
        db: &PgPool,
        event_type: String,
        user_id: Option<String>,
        email: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<String>,
        success: bool,
    ) {
        Self::log(
            db,
            &event_type,
            user_id,
            email,
            ip_address,
            user_agent,
            details,
            success,
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

pub type HandlerError = (axum::http::StatusCode, axum::Json<serde_json::Value>);

pub fn err(status: axum::http::StatusCode, message: &str) -> HandlerError {
    (status, axum::Json(serde_json::json!({ "error": message })))
}

pub fn err_internal(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg)
}

pub fn err_not_found(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::NOT_FOUND, msg)
}

pub fn err_forbidden(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::FORBIDDEN, msg)
}

pub fn err_bad_request(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::BAD_REQUEST, msg)
}

pub fn err_unauthorized(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::UNAUTHORIZED, msg)
}

pub fn err_conflict(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::CONFLICT, msg)
}

pub fn err_too_many_requests(msg: &str) -> HandlerError {
    err(axum::http::StatusCode::TOO_MANY_REQUESTS, msg)
}

pub fn err_created<T: serde::Serialize>(
    msg: &str,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::CREATED,
        axum::Json(serde_json::json!({ "message": msg })),
    )
}

pub type ServiceError = (axum::http::StatusCode, serde_json::Value);

#[must_use] 
pub fn svc_err(status: axum::http::StatusCode, message: &str) -> ServiceError {
    (status, serde_json::json!({ "error": message }))
}

#[must_use] 
pub fn svc_err_internal(msg: &str) -> ServiceError {
    svc_err(axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg)
}

#[must_use] 
pub fn svc_err_not_found(msg: &str) -> ServiceError {
    svc_err(axum::http::StatusCode::NOT_FOUND, msg)
}

#[must_use] 
pub fn svc_err_forbidden(msg: &str) -> ServiceError {
    svc_err(axum::http::StatusCode::FORBIDDEN, msg)
}

#[must_use] 
pub fn svc_err_bad_request(msg: &str) -> ServiceError {
    svc_err(axum::http::StatusCode::BAD_REQUEST, msg)
}
