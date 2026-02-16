use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::db::UserRole;

#[derive(Debug)]
pub enum RoleError {
    MissingToken,
    InvalidToken,
    InsufficientPermissions,
}

impl IntoResponse for RoleError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RoleError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing or invalid token"),
            RoleError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            RoleError::InsufficientPermissions => (
                StatusCode::FORBIDDEN,
                "Insufficient permissions for this operation",
            ),
        };
        (status, error_message).into_response()
    }
}

pub trait RoleValidator: Send + Sync {
    fn validate(&self, role: &UserRole) -> bool;
    #[must_use]
    fn get_error(&self) -> RoleError {
        RoleError::InsufficientPermissions
    }
}

#[derive(Debug)]
pub struct AdminValidator;
impl RoleValidator for AdminValidator {
    fn validate(&self, role: &UserRole) -> bool {
        role == &UserRole::BranchManager
            || role == &UserRole::CompanyManager
            || role == &UserRole::LogSmartAdmin
    }
}

#[derive(Debug)]
pub struct MemberValidator;
impl RoleValidator for MemberValidator {
    fn validate(&self, role: &UserRole) -> bool {
        role == &UserRole::Staff
            || role == &UserRole::BranchManager
            || role == &UserRole::CompanyManager
            || role == &UserRole::LogSmartAdmin
    }
}

#[derive(Debug)]
pub struct LogSmartAdminValidator;
impl RoleValidator for LogSmartAdminValidator {
    fn validate(&self, role: &UserRole) -> bool {
        role == &UserRole::LogSmartAdmin
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_role_error_into_response_missing_token() {
        let error = RoleError::MissingToken;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_role_error_into_response_invalid_token() {
        let error = RoleError::InvalidToken;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_role_error_into_response_insufficient_permissions() {
        let error = RoleError::InsufficientPermissions;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_validators_basic() {
        let member_validator = MemberValidator;
        let admin_validator = AdminValidator;
        let logsmart_admin_validator = LogSmartAdminValidator;

        let member_role = UserRole::Staff;
        let admin_role = UserRole::CompanyManager;
        let company_manager_role = UserRole::CompanyManager;
        let logsmart_admin_role = UserRole::LogSmartAdmin;

        assert!(member_validator.validate(&member_role));
        assert!(admin_validator.validate(&admin_role));
        assert!(admin_validator.validate(&company_manager_role));
        assert!(logsmart_admin_validator.validate(&logsmart_admin_role));
    }
}
