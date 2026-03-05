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

/// Validates that a string is a valid CSS color value.
/// Rejects values containing CSS syntax characters that could be used for injection.
///
/// Valid formats:
/// - Hex colors: #RGB, #RRGGBB, #RRGGBBAA
/// - RGB/RGBA: rgb(...), rgba(...)
/// - Named colors: red, blue, etc.
#[must_use]
pub fn is_valid_css_color(color: &str) -> bool {
    // Empty string is valid (means no custom color)
    if color.trim().is_empty() {
        return true;
    }

    let trimmed = color.trim();

    // Check for dangerous characters that could be used for CSS injection
    // Semicolon, curly braces, backslash (escape), and @ (at-rules)
    if trimmed.contains(';')
        || trimmed.contains('{')
        || trimmed.contains('}')
        || trimmed.contains('\\')
        || trimmed.contains('@')
    {
        return false;
    }

    // Hex color pattern: #RGB or #RRGGBB or #RRGGBBAA
    if regex::Regex::new(r"^#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?([0-9a-fA-F]{2})?$")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        return true;
    }

    // RGB/RGBA pattern: rgb(...) or rgba(...)
    if regex::Regex::new(r"^rgba?\s*\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*(,\s*[\d.]+\s*)?\)$")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        return true;
    }

    // Named colors: letters only (no spaces or special chars)
    if regex::Regex::new(r"^[a-zA-Z]+$")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        return true;
    }

    // Hex short format with alpha in older browsers
    if regex::Regex::new(r"^#[0-9a-fA-F]{4}$")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        return true;
    }

    // If none of the valid formats match, reject it
    false
}

/// Validates that a font family value is safe (no CSS injection characters).
/// Allows common font family names and safe CSS values.
pub fn is_valid_font_family(font_family: &str) -> bool {
    // Empty string is valid
    if font_family.trim().is_empty() {
        return true;
    }

    let trimmed = font_family.trim();

    // Check for dangerous characters that could be used for CSS injection
    if trimmed.contains(';')
        || trimmed.contains('{')
        || trimmed.contains('}')
        || trimmed.contains('\\')
        || trimmed.contains('@')
    {
        return false;
    }

    // Whitelist of safe font family values
    let safe_fonts = vec![
        "system-ui",
        "serif",
        "sans-serif",
        "monospace",
        "cursive",
        "fantasy",
        "georgia",
        "times",
        "courier",
        "verdana",
        "arial",
        "helvetica",
    ];

    // Check if it's in the safe list (case insensitive)
    let lower_font = trimmed.to_lowercase();
    if safe_fonts.iter().any(|f| f == &lower_font) {
        return true;
    }

    // Allow single quoted font names if they don't contain dangerous chars
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() > 2 {
        let inside = &trimmed[1..trimmed.len() - 1];
        // Font names with quotes can contain spaces but not dangerous chars
        return !inside.contains(';')
            && !inside.contains('{')
            && !inside.contains('}')
            && !inside.contains('\\')
            && !inside.contains('@');
    }

    // Allow double quoted font names
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
        let inside = &trimmed[1..trimmed.len() - 1];
        return !inside.contains(';')
            && !inside.contains('{')
            && !inside.contains('}')
            && !inside.contains('\\')
            && !inside.contains('@');
    }

    false
}

/// Validates that a text decoration value is safe (no CSS injection characters).
pub fn is_valid_text_decoration(text_decoration: &str) -> bool {
    // Empty string is valid
    if text_decoration.trim().is_empty() {
        return true;
    }

    let trimmed = text_decoration.trim();

    // Check for dangerous characters that could be used for CSS injection
    if trimmed.contains(';')
        || trimmed.contains('{')
        || trimmed.contains('}')
        || trimmed.contains('\\')
        || trimmed.contains('@')
    {
        return false;
    }

    // Whitelist of safe text decoration values
    let safe_decorations = ["none", "underline", "overline", "line-through", "blink"];

    // Check if it's in the safe list (case insensitive)
    let lower = trimmed.to_lowercase();
    safe_decorations.iter().any(|d| d == &lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_hex_colors() {
        // Valid hex colors
        assert!(is_valid_css_color("#FF0000")); // Red
        assert!(is_valid_css_color("#f00")); // Red short
        assert!(is_valid_css_color("#00FF00")); // Green
        assert!(is_valid_css_color("#0F0")); // Green short
        assert!(is_valid_css_color("#0000FF")); // Blue
        assert!(is_valid_css_color("#00F")); // Blue short
        assert!(is_valid_css_color("#FFFFFF")); // White
        assert!(is_valid_css_color("#FFF")); // White short
        assert!(is_valid_css_color("#000000")); // Black
        assert!(is_valid_css_color("#000")); // Black short
        assert!(is_valid_css_color("#FF0000FF")); // Red with full alpha
        assert!(is_valid_css_color("#F00F")); // Red short with alpha
        assert!(is_valid_css_color("#123456AA")); // With alpha
    }

    #[test]
    fn test_valid_rgb_colors() {
        // Valid RGB colors
        assert!(is_valid_css_color("rgb(255, 0, 0)")); // Red
        assert!(is_valid_css_color("rgb(0, 255, 0)")); // Green
        assert!(is_valid_css_color("rgb(0, 0, 255)")); // Blue
        assert!(is_valid_css_color("rgb(255,0,0)")); // No spaces
        assert!(is_valid_css_color("rgb(  255  ,  0  ,  0  )")); // Extra spaces
    }

    #[test]
    fn test_valid_rgba_colors() {
        // Valid RGBA colors
        assert!(is_valid_css_color("rgba(255, 0, 0, 1)")); // Red fully opaque
        assert!(is_valid_css_color("rgba(255, 0, 0, 0.5)")); // Red semi-transparent
        assert!(is_valid_css_color("rgba(0, 255, 0, 0)")); // Green fully transparent
        assert!(is_valid_css_color("rgba(0,0,255,0.75)")); // No spaces
        assert!(is_valid_css_color("rgba(  100  ,  100  ,  100  ,  0.5  )")); // Extra spaces
    }

    #[test]
    fn test_valid_named_colors() {
        // Valid named colors
        assert!(is_valid_css_color("red"));
        assert!(is_valid_css_color("blue"));
        assert!(is_valid_css_color("green"));
        assert!(is_valid_css_color("white"));
        assert!(is_valid_css_color("black"));
        assert!(is_valid_css_color("transparent"));
        assert!(is_valid_css_color("darkred"));
        assert!(is_valid_css_color("lightblue"));
    }

    #[test]
    fn test_empty_string_is_valid() {
        // Empty string is valid (means no custom color)
        assert!(is_valid_css_color(""));
        assert!(is_valid_css_color("   ")); // Whitespace only
    }

    #[test]
    fn test_malicious_semicolon_injection() {
        // CSS injection via semicolon
        assert!(!is_valid_css_color("red;color:blue"));
        assert!(!is_valid_css_color("#FF0000;display:none"));
        assert!(!is_valid_css_color("rgb(255,0,0);opacity:0"));
        assert!(!is_valid_css_color("red;font-size:100px"));
    }

    #[test]
    fn test_malicious_curly_brace_injection() {
        // CSS injection via curly braces
        assert!(!is_valid_css_color("red{display:none}"));
        assert!(!is_valid_css_color("#FF0000{color:blue}"));
        assert!(!is_valid_css_color("rgb(255,0,0){font-size:100px}"));
    }

    #[test]
    fn test_malicious_at_rule_injection() {
        // CSS injection via @-rules
        assert!(!is_valid_css_color("@import url('evil.css')"));
        assert!(!is_valid_css_color("red@keyframes"));
        assert!(!is_valid_css_color("#FF0000@media"));
    }

    #[test]
    fn test_malicious_backslash_injection() {
        // CSS escape sequences
        assert!(!is_valid_css_color("red\\"));
        assert!(!is_valid_css_color("red\\000041"));
        assert!(!is_valid_css_color("#FF0000\\20display\\3Anone"));
    }

    #[test]
    fn test_malicious_comment_injection() {
        // Note: Comments don't have the injection characters, but testing edge cases
        assert!(!is_valid_css_color("red/**/color:blue")); // Comment with extra chars won't match patterns
        assert!(!is_valid_css_color("rgb(255,0,0)/*comment*/")); // This will fail because of special chars
    }

    #[test]
    fn test_invalid_color_formats() {
        // Invalid color formats
        assert!(!is_valid_css_color("123456")); // No # for hex
        assert!(!is_valid_css_color("#GGGGGG")); // Invalid hex characters
        assert!(!is_valid_css_color("#FF")); // Too few hex digits (2)
        assert!(!is_valid_css_color("#FF00000")); // Invalid hex length (7 digits)
        assert!(!is_valid_css_color("rgb(-1, 0, 0)")); // Negative values (starts with -)
        assert!(!is_valid_css_color("rgb(255, 0)")); // Missing parameter
        assert!(!is_valid_css_color("rgb(255 0 0)")); // Space separator instead of comma
        assert!(!is_valid_css_color("hsl(120, 100%, 50%)")); // HSL not supported in our validation
        assert!(!is_valid_css_color("rgba 255 0 0 1")); // Invalid syntax
        assert!(!is_valid_css_color("red blue")); // Multiple colors
        assert!(!is_valid_css_color("red!")); // Special characters
    }

    #[test]
    fn test_case_insensitive_colors() {
        // Color names should be case insensitive in CSS, but our validator accepts any letters
        assert!(is_valid_css_color("RED"));
        assert!(is_valid_css_color("Red"));
        assert!(is_valid_css_color("rEd"));
        assert!(is_valid_css_color("#ff0000")); // Hex lowercase
        assert!(is_valid_css_color("#FF0000")); // Hex uppercase
        assert!(is_valid_css_color("#Ff00Ff")); // Hex mixed case
    }

    #[test]
    fn test_valid_font_families() {
        // Safe font family values
        assert!(is_valid_font_family("system-ui"));
        assert!(is_valid_font_family("serif"));
        assert!(is_valid_font_family("sans-serif"));
        assert!(is_valid_font_family("monospace"));
        assert!(is_valid_font_family("cursive"));
        assert!(is_valid_font_family("fantasy"));
        assert!(is_valid_font_family("georgia"));
        assert!(is_valid_font_family("times"));
        assert!(is_valid_font_family("courier"));
        assert!(is_valid_font_family("verdana"));
        assert!(is_valid_font_family("arial"));
        assert!(is_valid_font_family("helvetica"));
        assert!(is_valid_font_family("Georgia")); // Case insensitive
        assert!(is_valid_font_family("ARIAL"));
        assert!(is_valid_font_family("")); // Empty is valid
        assert!(is_valid_font_family("   ")); // Whitespace only is valid
    }

    #[test]
    fn test_quoted_font_families() {
        // Quoted font families are allowed
        assert!(is_valid_font_family("'Custom Font'"));
        assert!(is_valid_font_family("'Times New Roman'"));
        assert!(is_valid_font_family("\"Custom Font\""));
        assert!(is_valid_font_family("\"Courier New\""));
    }

    #[test]
    fn test_malicious_font_families() {
        // Font families with injection characters should be rejected
        assert!(!is_valid_font_family("serif;color:red"));
        assert!(!is_valid_font_family("arial{display:none}"));
        assert!(!is_valid_font_family("times\\000041"));
        assert!(!is_valid_font_family("@import"));
        assert!(!is_valid_font_family("'Custom';display:none")); // Injection in quoted
    }

    #[test]
    fn test_valid_text_decorations() {
        // Safe text decoration values
        assert!(is_valid_text_decoration("none"));
        assert!(is_valid_text_decoration("underline"));
        assert!(is_valid_text_decoration("overline"));
        assert!(is_valid_text_decoration("line-through"));
        assert!(is_valid_text_decoration("blink"));
        assert!(is_valid_text_decoration("None")); // Case insensitive
        assert!(is_valid_text_decoration("UNDERLINE"));
        assert!(is_valid_text_decoration("")); // Empty is valid
        assert!(is_valid_text_decoration("   ")); // Whitespace only is valid
    }

    #[test]
    fn test_malicious_text_decorations() {
        // Text decorations with injection characters should be rejected
        assert!(!is_valid_text_decoration("none;color:red"));
        assert!(!is_valid_text_decoration("underline{display:none}"));
        assert!(!is_valid_text_decoration("line-through\\000041"));
        assert!(!is_valid_text_decoration("@keyframes"));
        assert!(!is_valid_text_decoration("invalid-value")); // Not a valid decoration
    }
}
