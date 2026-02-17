use back_end::db::{Branch, Company, Invitation, UserRecord, UserRole};

#[test]
fn test_user_role_company_manager() {
    let role = UserRole::CompanyManager;
    assert_eq!(role.to_string(), "company_manager");
}

#[test]
fn test_user_role_staff() {
    let role = UserRole::Staff;
    assert_eq!(role.to_string(), "staff");
}

#[test]
fn test_user_role_branch_manager() {
    let role = UserRole::BranchManager;
    assert_eq!(role.to_string(), "branch_manager");
}

#[test]
fn test_user_role_logsmart_admin() {
    let role = UserRole::LogSmartAdmin;
    assert_eq!(role.to_string(), "logsmart_admin");
}

#[test]
fn test_user_role_from_str_company_manager() {
    let role: UserRole = "company_manager".parse().unwrap();
    assert_eq!(role, UserRole::CompanyManager);
}

#[test]
fn test_user_role_from_str_staff() {
    let role: UserRole = "staff".parse().unwrap();
    assert_eq!(role, UserRole::Staff);
}

#[test]
fn test_user_role_from_str_branch_manager() {
    let role: UserRole = "branch_manager".parse().unwrap();
    assert_eq!(role, UserRole::BranchManager);
}

#[test]
fn test_user_role_from_str_invalid() {
    let result: Result<UserRole, _> = "invalid".parse();
    assert!(result.is_err());
}

#[test]
fn test_user_get_role_company_manager() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}

#[test]
fn test_branch_creation() {
    let branch = Branch {
        id: "branch1".to_string(),
        company_id: "company1".to_string(),
        name: "Main Office".to_string(),
        address: "123 Main St".to_string(),
        created_at: chrono::Utc::now(),
    };
    assert_eq!(branch.id, "branch1");
    assert_eq!(branch.company_id, "company1");
    assert_eq!(branch.name, "Main Office");
    assert_eq!(branch.address, "123 Main St");
}

#[test]
fn test_branch_equality() {
    let created_at = chrono::Utc::now();
    let branch1 = Branch {
        id: "branch1".to_string(),
        company_id: "company1".to_string(),
        name: "Main Office".to_string(),
        address: "123 Main St".to_string(),
        created_at,
    };
    let branch2 = Branch {
        id: "branch1".to_string(),
        company_id: "company1".to_string(),
        name: "Main Office".to_string(),
        address: "123 Main St".to_string(),
        created_at,
    };
    assert_eq!(branch1, branch2);
}

#[test]
fn test_branch_serialization() {
    let branch = Branch {
        id: "branch1".to_string(),
        company_id: "company1".to_string(),
        name: "Main Office".to_string(),
        address: "123 Main St".to_string(),
        created_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&branch).unwrap();
    assert!(json.contains("branch1"));
    assert!(json.contains("Main Office"));
}

#[test]
fn test_branch_deserialization() {
    let json = r#"{
        "id": "branch1",
        "company_id": "company1",
        "name": "Main Office",
        "address": "123 Main St",
        "created_at": "2024-01-01T00:00:00Z"
    }"#;
    let branch: Branch = serde_json::from_str(json).unwrap();
    assert_eq!(branch.id, "branch1");
    assert_eq!(branch.name, "Main Office");
}

#[test]
fn test_user_get_role_staff() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.get_role(), UserRole::Staff);
}

#[test]
fn test_user_is_company_manager_true() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "manager@example.com".to_string(),
        first_name: "Company".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(user.is_company_manager());
}

#[test]
fn test_user_is_company_manager_false() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "user@example.com".to_string(),
        first_name: "Staff".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_company_manager());
}

#[test]
fn test_user_is_branch_manager_true() {
    let user = UserRecord {
        id: "user3".to_string(),
        email: "bm@example.com".to_string(),
        first_name: "Branch".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::BranchManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(user.is_branch_manager());
}

#[test]
fn test_user_is_staff_true() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "user@example.com".to_string(),
        first_name: "Staff".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(user.is_staff());
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
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::CompanyManager,
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
    assert_eq!(user.branch_id, Some("branch1".to_string()));
    assert_eq!(user.company_name, None);
    assert_eq!(user.role, UserRole::CompanyManager);
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
        role: UserRole::Staff,
        branch_id: Some("branch1".to_string()),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        accepted_at: None,
        cancelled_at: None,
    };
    assert_eq!(invitation.id, "invite1");
    assert_eq!(invitation.company_id, "company1");
    assert_eq!(invitation.email, "newuser@example.com");
    assert_eq!(invitation.token, "token123");
    assert_eq!(invitation.role, UserRole::Staff);
    assert_eq!(invitation.branch_id, Some("branch1".to_string()));
    assert_eq!(invitation.accepted_at, None);
}

#[test]
fn test_invitation_accepted() {
    let invitation = Invitation {
        id: "invite1".to_string(),
        company_id: "company1".to_string(),
        email: "newuser@example.com".to_string(),
        token: "token123".to_string(),
        role: UserRole::Staff,
        branch_id: None,
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
        branch_id: None,
        company_name: None,
        role: UserRole::LogSmartAdmin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert_eq!(user.company_id, None);
    assert_eq!(user.branch_id, None);
}

#[test]
fn test_user_role_serialization() {
    let role = UserRole::CompanyManager;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"company_manager\"");
}

#[test]
fn test_user_role_deserialization() {
    let json = "\"staff\"";
    let role: UserRole = serde_json::from_str(json).unwrap();
    assert_eq!(role, UserRole::Staff);
}

#[test]
fn test_user_can_manage_company() {
    let company_manager = UserRecord {
        id: "user1".to_string(),
        email: "manager@example.com".to_string(),
        first_name: "Company".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(company_manager.can_manage_company());

    let logsmart_admin = UserRecord {
        id: "user2".to_string(),
        email: "admin@logsmart.app".to_string(),
        first_name: "LogSmart".to_string(),
        last_name: "Admin".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: None,
        branch_id: None,
        company_name: None,
        role: UserRole::LogSmartAdmin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(logsmart_admin.can_manage_company());

    let staff = UserRecord {
        id: "user3".to_string(),
        email: "staff@example.com".to_string(),
        first_name: "Staff".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!staff.can_manage_company());
}

#[test]
fn test_user_can_manage_branch() {
    let company_manager = UserRecord {
        id: "user1".to_string(),
        email: "manager@example.com".to_string(),
        first_name: "Company".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(company_manager.can_manage_branch());

    let branch_manager = UserRecord {
        id: "user2".to_string(),
        email: "bm@example.com".to_string(),
        first_name: "Branch".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::BranchManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(branch_manager.can_manage_branch());

    let staff = UserRecord {
        id: "user3".to_string(),
        email: "staff@example.com".to_string(),
        first_name: "Staff".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!staff.can_manage_branch());
}

#[test]
fn test_user_is_readonly_hq_true() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "hq@example.com".to_string(),
        first_name: "HQ".to_string(),
        last_name: "Staff".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(user.is_readonly_hq());
}

#[test]
fn test_user_is_readonly_hq_false_staff_with_branch() {
    let user = UserRecord {
        id: "user2".to_string(),
        email: "staff@example.com".to_string(),
        first_name: "Staff".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}

#[test]
fn test_user_is_readonly_hq_false_company_manager() {
    let user = UserRecord {
        id: "user3".to_string(),
        email: "manager@example.com".to_string(),
        first_name: "Company".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}

#[test]
fn test_user_is_readonly_hq_false_branch_manager() {
    let user = UserRecord {
        id: "user4".to_string(),
        email: "bm@example.com".to_string(),
        first_name: "Branch".to_string(),
        last_name: "Manager".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::BranchManager,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}

#[test]
fn test_user_is_readonly_hq_false_logsmart_admin() {
    let user = UserRecord {
        id: "user5".to_string(),
        email: "admin@logsmart.app".to_string(),
        first_name: "LogSmart".to_string(),
        last_name: "Admin".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: None,
        branch_id: None,
        company_name: None,
        role: UserRole::LogSmartAdmin,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}

#[test]
fn test_user_is_readonly_hq_false_no_company() {
    let user = UserRecord {
        id: "user6".to_string(),
        email: "nocompany@example.com".to_string(),
        first_name: "No".to_string(),
        last_name: "Company".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: None,
        branch_id: Some("branch1".to_string()),
        company_name: None,
        role: UserRole::Staff,
        created_at: chrono::Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };
    assert!(!user.is_readonly_hq());
}
