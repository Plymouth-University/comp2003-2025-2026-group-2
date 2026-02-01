use back_end::db::{Company, Invitation, UserRecord, UserRole};

#[test]
fn test_user_role_admin() {
    let role = UserRole::Admin;
    assert_eq!(role.to_string(), "admin");
}

#[test]
fn test_user_role_member() {
    let role = UserRole::Member;
    assert_eq!(role.to_string(), "member");
}

#[test]
fn test_user_role_from_str_admin() {
    let role: UserRole = "admin".parse().unwrap();
    assert_eq!(role, UserRole::Admin);
}

#[test]
fn test_user_role_from_str_member() {
    let role: UserRole = "member".parse().unwrap();
    assert_eq!(role, UserRole::Member);
}

#[test]
fn test_user_role_from_str_invalid() {
    let result: Result<UserRole, _> = "invalid".parse();
    assert!(result.is_err());
}

#[test]
fn test_user_get_role_admin() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        company_name: None,
        role: UserRole::Admin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.get_role(), UserRole::Admin);
}

#[test]
fn test_user_get_role_member() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        company_name: None,
        role: UserRole::Member,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.get_role(), UserRole::Member);
}

#[test]
fn test_user_is_admin_true() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "admin@example.com".to_string(),
        first_name: "Admin".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        company_name: None,
        role: UserRole::Admin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(user.is_admin());
}

#[test]
fn test_user_is_admin_false() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "user@example.com".to_string(),
        first_name: "Regular".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        company_name: None,
        role: UserRole::Member,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_admin());
}

#[test]
fn test_user_creation() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "user@example.com".to_string(),
        first_name: "Regular".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        company_name: None,
        role: UserRole::Admin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.id, "user2");
    assert_eq!(user.email, "user@example.com");
    assert_eq!(user.first_name, "Regular");
    assert_eq!(user.last_name, "User");
    assert_eq!(user.password_hash, Some("hash".to_string()));
    assert_eq!(user.company_id, Some("company1".to_string()));
    assert_eq!(user.company_name, None);
    assert_eq!(user.role, UserRole::Admin);
}

#[test]
fn test_company_creation() {
    let company = Company {
        id: "company1".to_string(),
        name: "Test Company".to_string(),
        address: "123 Main St".to_string(),
        created_at: chrono::Utc::now(),
    };
    assert_eq!(company.id, "company1");
    assert_eq!(company.name, "Test Company");
    assert_eq!(company.address, "123 Main St");
}

#[test]
fn test_invitation_creation() {
    let invitation = Invitation {
        id: "invite1".to_string(),
        company_id: "company1".to_string(),
        email: "newuser@example.com".to_string(),
        token: "token123".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        accepted_at: None,
        cancelled_at: None,
    };
    assert_eq!(invitation.id, "invite1");
    assert_eq!(invitation.company_id, "company1");
    assert_eq!(invitation.email, "newuser@example.com");
    assert_eq!(invitation.token, "token123");
    assert_eq!(invitation.accepted_at, None);
}

#[test]
fn test_invitation_accepted() {
    let invitation = Invitation {
        id: "invite1".to_string(),
        company_id: "company1".to_string(),
        email: "newuser@example.com".to_string(),
        token: "token123".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        accepted_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        cancelled_at: None,
    };
    assert!(invitation.accepted_at.is_some());
}

#[test]
fn test_user_without_company() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "admin@logsmart.app".to_string(),
        first_name: "Admin".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: None,
        company_name: None,
        role: UserRole::LogSmartAdmin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.company_id, None);
}

#[test]
fn test_user_role_serialization() {
    let role = UserRole::Admin;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"admin\"");
}

#[test]
fn test_user_role_deserialization() {
    let json = "\"member\"";
    let role: UserRole = serde_json::from_str(json).unwrap();
    assert_eq!(role, UserRole::Member);
}
