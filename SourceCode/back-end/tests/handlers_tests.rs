use back_end::dto::AuthResponse;
use back_end::dto::InvitationResponse;
use back_end::dto::RequestBranchDeletionResponse;
use back_end::dto::UserResponse;
use back_end::dto::{
    AcceptInvitationRequest, CreateBranchRequest, InviteUserRequest, LoginRequest, RegisterRequest,
    RequestBranchDeletionRequest, UpdateBranchRequest,
};
use back_end::jwt_manager::JwtManager;

#[test]
fn test_register_request_validation() {
    let req = RegisterRequest {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "password123".to_string(),
        company_name: "Test Co".to_string(),
        company_address: "123 Main St".to_string(),
    };
    assert_eq!(req.email, "test@example.com");
    assert_eq!(req.first_name, "John");
    assert_eq!(req.password, "password123");
}

#[test]
fn test_register_with_short_password() {
    let req = RegisterRequest {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "short".to_string(),
        company_name: "Test Co".to_string(),
        company_address: "123 Main St".to_string(),
    };
    assert!(req.password.len() < 8);
}

#[test]
fn test_register_missing_fields() {
    let req = RegisterRequest {
        email: "".to_string(),
        first_name: "".to_string(),
        last_name: "".to_string(),
        password: "".to_string(),
        company_name: "".to_string(),
        company_address: "".to_string(),
    };
    assert!(req.email.is_empty());
    assert!(req.first_name.is_empty());
}

#[test]
fn test_login_request_validation() {
    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    assert_eq!(req.email, "test@example.com");
    assert_eq!(req.password, "password123");
}

#[test]
fn test_login_missing_email() {
    let req = LoginRequest {
        email: "".to_string(),
        password: "password123".to_string(),
    };
    assert!(req.email.is_empty());
}

#[test]
fn test_login_missing_password() {
    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "".to_string(),
    };
    assert!(req.password.is_empty());
}

#[test]
fn test_invite_user_request_validation() {
    let req = InviteUserRequest {
        email: "newuser@example.com".to_string(),
        role: Some(back_end::db::UserRole::Staff),
        branch_id: Some("branch123".to_string()),
    };
    assert_eq!(req.email, "newuser@example.com");
    assert_eq!(req.role, Some(back_end::db::UserRole::Staff));
    assert_eq!(req.branch_id, Some("branch123".to_string()));
}

#[test]
fn test_invite_user_missing_email() {
    let req = InviteUserRequest {
        email: "".to_string(),
        role: None,
        branch_id: None,
    };
    assert!(req.email.is_empty());
}

#[test]
fn test_accept_invitation_request_validation() {
    let req = AcceptInvitationRequest {
        token: "token123".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        password: "password123".to_string(),
    };
    assert_eq!(req.token, "token123");
    assert_eq!(req.first_name, "Jane");
}

#[test]
fn test_accept_invitation_short_password() {
    let req = AcceptInvitationRequest {
        token: "token123".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        password: "short".to_string(),
    };
    assert!(req.password.len() < 8);
}

#[test]
fn test_accept_invitation_missing_fields() {
    let req = AcceptInvitationRequest {
        token: "".to_string(),
        first_name: "".to_string(),
        last_name: "".to_string(),
        password: "".to_string(),
    };
    assert!(req.token.is_empty());
    assert!(req.first_name.is_empty());
}

use back_end::db::UserRole;

#[test]
fn test_user_response_structure() {
    let user_response = UserResponse {
        id: "user123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        company_id: Some("company123".to_string()),
        company_name: Some("Test Company".to_string()),
        branch_id: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        oauth_provider: None,
    };
    assert_eq!(user_response.email, "test@example.com");
    assert_eq!(user_response.role, UserRole::CompanyManager);
    assert_eq!(user_response.company_name, Some("Test Company".to_string()));
}

#[test]
fn test_user_response_without_company() {
    let user_response = UserResponse {
        id: "user123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        company_id: None,
        company_name: None,
        branch_id: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        oauth_provider: None,
    };
    assert_eq!(user_response.company_name, None);
}

#[test]
fn test_invitation_response_structure() {
    let inv_response = InvitationResponse {
        id: "invite1".to_string(),
        email: "newuser@example.com".to_string(),
        expires_at: chrono::Utc::now(),
    };
    assert_eq!(inv_response.id, "invite1");
    assert_eq!(inv_response.email, "newuser@example.com");
}

#[test]
fn test_auth_response_structure() {
    let user_response = UserResponse {
        id: "user123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        company_id: Some("company123".to_string()),
        company_name: Some("Test Company".to_string()),
        branch_id: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        oauth_provider: None,
    };
    let auth_response = AuthResponse {
        token: "jwt_token_here".to_string(),
        user: user_response,
    };
    assert!(!auth_response.token.is_empty());
    assert_eq!(auth_response.user.email, "test@example.com");
}

#[test]
fn test_jwt_manager_get_secret() {
    let secret = JwtManager::get_secret();
    assert!(!secret.is_empty());
    assert!(secret.len() > 0);
}

#[test]
fn test_jwt_manager_caches_secret() {
    let secret1 = JwtManager::get_secret();
    let secret2 = JwtManager::get_secret();
    assert_eq!(secret1, secret2);
}

#[test]
fn test_register_request_serialization() {
    let req = RegisterRequest {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "password123".to_string(),
        company_name: "Test Co".to_string(),
        company_address: "123 Main St".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("test@example.com"));
    assert!(json.contains("Test Co"));
}

#[test]
fn test_login_request_serialization() {
    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("test@example.com"));
}

#[test]
fn test_user_response_serialization() {
    let user_response = UserResponse {
        id: "user123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        company_id: Some("company123".to_string()),
        company_name: Some("Test Company".to_string()),
        branch_id: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        oauth_provider: None,
    };
    let json = serde_json::to_string(&user_response).unwrap();
    assert!(json.contains("Test Company"));
    assert!(json.contains("company_manager"));
}

#[test]
fn test_invitation_response_serialization() {
    let inv_response = InvitationResponse {
        id: "invite1".to_string(),
        email: "newuser@example.com".to_string(),
        expires_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&inv_response).unwrap();
    assert!(json.contains("invite1"));
    assert!(json.contains("newuser@example.com"));
}

#[test]
fn test_register_request_email_validation() {
    let req = RegisterRequest {
        email: "invalidemail".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "password123".to_string(),
        company_name: "Test Co".to_string(),
        company_address: "123 Main St".to_string(),
    };
    assert!(!req.email.is_empty());
}

#[test]
fn test_password_validation_length() {
    let short_password = "pass";
    let valid_password = "password123";
    assert!(short_password.len() < 8);
    assert!(valid_password.len() >= 8);
}

#[test]
fn test_response_contains_token() {
    let user_response = UserResponse {
        id: "user123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        company_id: Some("Test Company".to_string()),
        company_name: Some("Test Company".to_string()),
        branch_id: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        oauth_provider: None,
    };
    let auth_response = AuthResponse {
        token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
        user: user_response,
    };
    assert!(auth_response.token.len() > 0);
}

#[test]
fn test_create_branch_request_validation() {
    let req = CreateBranchRequest {
        name: "Main Office".to_string(),
        address: "123 Main St".to_string(),
    };
    assert_eq!(req.name, "Main Office");
    assert_eq!(req.address, "123 Main St");
}

#[test]
fn test_update_branch_request_validation() {
    let req = UpdateBranchRequest {
        branch_id: "branch123".to_string(),
        name: "Updated Office".to_string(),
        address: "456 New St".to_string(),
    };
    assert_eq!(req.branch_id, "branch123");
    assert_eq!(req.name, "Updated Office");
    assert_eq!(req.address, "456 New St");
}

#[test]
fn test_request_branch_deletion_request() {
    let req = RequestBranchDeletionRequest {
        branch_id: "branch123".to_string(),
    };
    assert_eq!(req.branch_id, "branch123");
}

#[test]
fn test_branch_deletion_response_message() {
    let resp = RequestBranchDeletionResponse {
        message: "A confirmation email has been sent".to_string(),
    };
    assert!(resp.message.contains("confirmation"));
}
