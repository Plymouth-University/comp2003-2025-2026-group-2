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
