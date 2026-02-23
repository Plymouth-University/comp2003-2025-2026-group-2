use axum::response::IntoResponse;
use back_end::security::{AdminValidator, LogSmartAdminValidator, MemberValidator, RoleError, RoleValidator};

#[test]
fn test_role_error_into_response_missing_token() {
    let error = RoleError::MissingToken;
    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_role_error_into_response_invalid_token() {
    let error = RoleError::InvalidToken;
    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_role_error_into_response_insufficient_permissions() {
    let error = RoleError::InsufficientPermissions;
    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
}

#[test]
fn test_validators_basic() {
    let member_validator = MemberValidator;
    let admin_validator = AdminValidator;
    let logsmart_admin_validator = LogSmartAdminValidator;

    let member_role = back_end::db::UserRole::Staff;
    let admin_role = back_end::db::UserRole::CompanyManager;
    let company_manager_role = back_end::db::UserRole::CompanyManager;
    let logsmart_admin_role = back_end::db::UserRole::LogSmartAdmin;

    assert!(member_validator.validate(&member_role));
    assert!(admin_validator.validate(&admin_role));
    assert!(admin_validator.validate(&company_manager_role));
    assert!(logsmart_admin_validator.validate(&logsmart_admin_role));
}
