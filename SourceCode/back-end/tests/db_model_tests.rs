use back_end::db::{
    Company, Invitation, Passkey, PasskeySession, SecurityLog, UserRecord, UserRole,
};
use chrono::{Duration, Utc};

fn create_test_user_record() -> UserRecord {
    UserRecord {
        id: "user1".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::Staff,
        created_at: Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
        profile_picture_id: None,
        company_deleted_at: None,
    }
}

fn create_test_invitation() -> Invitation {
    Invitation {
        id: "inv1".to_string(),
        company_id: "company1".to_string(),
        email: "test@example.com".to_string(),
        token: "token123".to_string(),
        role: UserRole::Staff,
        branch_id: None,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        accepted_at: None,
        cancelled_at: None,
    }
}

fn create_test_security_log() -> SecurityLog {
    SecurityLog {
        id: "log1".to_string(),
        event_type: "test_event".to_string(),
        user_id: Some("user1".to_string()),
        email: Some("test@example.com".to_string()),
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Test Browser".to_string()),
        details: Some("Test details".to_string()),
        success: true,
        created_at: Utc::now(),
        actor_role: Some("logsmart_admin".to_string()),
        company_id: Some("company1-uuid".to_string()),
        request_method: None,
        request_path: None,
        target_email: None,
        target_user_id: None,
    }
}

fn create_test_passkey() -> Passkey {
    Passkey {
        id: "pk1".to_string(),
        user_id: "user1".to_string(),
        credential_id: "cred123".to_string(),
        public_key: "public_key_data".to_string(),
        counter: 0,
        name: "Test Passkey".to_string(),
        created_at: Utc::now(),
        last_used_at: None,
    }
}

fn create_test_passkey_session() -> PasskeySession {
    PasskeySession {
        id: "session1".to_string(),
        session_type: "test".to_string(),
        user_id: Some("user1".to_string()),
        challenge: "challenge123".to_string(),
        meta: None,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::minutes(5),
    }
}

#[test]
fn test_user_role_company_manager_string_conversion() {
    let role = UserRole::CompanyManager;
    assert_eq!(role.to_string(), "company_manager");
}

#[test]
fn test_user_role_branch_manager_string_conversion() {
    let role = UserRole::BranchManager;
    assert_eq!(role.to_string(), "branch_manager");
}

#[test]
fn test_user_role_staff_string_conversion() {
    let role = UserRole::Staff;
    assert_eq!(role.to_string(), "staff");
}

#[test]
fn test_user_role_logsmart_admin_string_conversion() {
    let role = UserRole::LogSmartAdmin;
    assert_eq!(role.to_string(), "logsmart_admin");
}

#[test]
fn test_user_role_from_str_branch_manager() {
    let role: UserRole = "branch_manager".parse().unwrap();
    assert_eq!(role, UserRole::BranchManager);
}

#[test]
fn test_user_role_from_str_staff() {
    let role: UserRole = "staff".parse().unwrap();
    assert_eq!(role, UserRole::Staff);
}

#[test]
fn test_user_role_from_str_company_manager() {
    let role: UserRole = "company_manager".parse().unwrap();
    assert_eq!(role, UserRole::CompanyManager);
}

#[test]
fn test_user_role_from_str_logsmart_admin() {
    let role: UserRole = "logsmart_admin".parse().unwrap();
    assert_eq!(role, UserRole::LogSmartAdmin);
}

#[test]
fn test_user_role_from_str_invalid() {
    let result: Result<UserRole, _> = "invalid_role".parse();
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Unknown role: invalid_role"));
}

#[test]
fn test_user_role_case_sensitivity() {
    assert!("ADMIN".parse::<UserRole>().is_err());
    assert!("Admin".parse::<UserRole>().is_err());
    assert!("COMPANY_MANAGER".parse::<UserRole>().is_err());
    assert!("company_manager".parse::<UserRole>().is_ok());
}

#[test]
fn test_user_role_serde_compatibility() {
    let role = UserRole::BranchManager;
    let serialized = serde_json::to_string(&role).unwrap();
    assert!(serialized.contains("\"branch_manager\""));

    let deserialized: UserRole = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, UserRole::BranchManager);
}

#[test]
fn test_user_record_role_methods() {
    let user = UserRecord {
        id: "user1".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company1".to_string()),
        branch_id: None,
        company_name: None,
        role: UserRole::CompanyManager,
        created_at: Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
        profile_picture_id: None,
        company_deleted_at: None,
    };

    assert!(!user.is_branch_manager());
    assert!(!user.is_logsmart_admin());
    assert!(user.is_company_manager());
    assert!(user.can_manage_company());
}

#[test]
fn test_user_record_member_role_methods() {
    let user = UserRecord {
        role: UserRole::Staff,
        ..create_test_user_record()
    };

    assert!(!user.is_branch_manager());
    assert!(user.is_staff());
    assert!(!user.is_logsmart_admin());
    assert!(!user.is_company_manager());
    assert!(!user.can_manage_company());
}

#[test]
fn test_user_record_logsmart_admin_role_methods() {
    let user = UserRecord {
        role: UserRole::LogSmartAdmin,
        ..create_test_user_record()
    };

    assert!(!user.is_branch_manager());
    assert!(!user.is_staff());
    assert!(user.is_logsmart_admin());
    assert!(!user.is_company_manager());
    assert!(user.can_manage_company());
}

#[test]
fn test_user_record_oauth_user() {
    let user = UserRecord {
        password_hash: None,
        oauth_provider: Some("google".to_string()),
        oauth_subject: Some("google_subject_123".to_string()),
        oauth_picture: Some("https://example.com/pic.jpg".to_string()),
        ..create_test_user_record()
    };

    assert!(user.password_hash.is_none());
    assert_eq!(user.oauth_provider, Some("google".to_string()));
    assert_eq!(user.oauth_subject, Some("google_subject_123".to_string()));
    assert_eq!(
        user.oauth_picture,
        Some("https://example.com/pic.jpg".to_string())
    );
}

#[test]
fn test_user_record_deleted_user() {
    let user = UserRecord {
        deleted_at: Some(Utc::now()),
        ..create_test_user_record()
    };

    assert!(user.deleted_at.is_some());
}

#[test]
fn test_user_record_partial_company_info() {
    let user = UserRecord {
        company_id: Some("company1".to_string()),
        company_name: Some("Test Company".to_string()),
        ..create_test_user_record()
    };

    assert_eq!(user.company_id, Some("company1".to_string()));
    assert_eq!(user.company_name, Some("Test Company".to_string()));
}

#[test]
fn test_user_record_no_company_info() {
    let user = UserRecord {
        company_id: None,
        company_name: None,
        ..create_test_user_record()
    };

    assert!(user.company_id.is_none());
    assert!(user.company_name.is_none());
    assert!(!user.can_manage_company());
}

#[test]
fn test_company_creation() {
    let company = Company::new().with_name_and_address("Test Company", "123 Test St");

    assert_eq!(company.name, "Test Company");
    assert_eq!(company.address, "123 Test St");
    assert_eq!(company.logo_id, None);
    assert_eq!(company.data_exported_at, None);
    assert_eq!(company.deleted_at, None);
}

#[test]
fn test_invitation_creation() {
    let now = Utc::now();
    let expires_at = now + Duration::hours(24);
    let invitation = Invitation {
        id: "inv1".to_string(),
        company_id: "company1".to_string(),
        email: "test@example.com".to_string(),
        token: "token123".to_string(),
        role: UserRole::Staff,
        branch_id: None,
        created_at: now,
        expires_at,
        accepted_at: None,
        cancelled_at: None,
    };

    assert_eq!(invitation.id, "inv1");
    assert_eq!(invitation.company_id, "company1");
    assert_eq!(invitation.email, "test@example.com");
    assert_eq!(invitation.token, "token123");
    assert_eq!(invitation.created_at, now);
    assert_eq!(invitation.expires_at, expires_at);
    assert!(invitation.accepted_at.is_none());
    assert!(invitation.cancelled_at.is_none());
}

#[test]
fn test_invitation_accepted() {
    let accepted_at = Some(Utc::now());
    let invitation = Invitation {
        accepted_at,
        ..create_test_invitation()
    };

    assert!(invitation.accepted_at.is_some());
}

#[test]
fn test_invitation_cancelled() {
    let cancelled_at = Some(Utc::now());
    let invitation = Invitation {
        cancelled_at,
        ..create_test_invitation()
    };

    assert!(invitation.cancelled_at.is_some());
}

#[test]
fn test_invitation_expired_check() {
    let now = Utc::now();
    let expired_invitation = Invitation {
        expires_at: now - Duration::hours(1),
        ..create_test_invitation()
    };

    let valid_invitation = Invitation {
        expires_at: now + Duration::hours(1),
        ..create_test_invitation()
    };

    assert!(expired_invitation.expires_at < now);
    assert!(valid_invitation.expires_at > now);
}

#[test]
fn test_security_log_creation() {
    let now = Utc::now();
    let log = SecurityLog {
        id: "log1".to_string(),
        event_type: "login".to_string(),
        user_id: Some("user1".to_string()),
        email: Some("test@example.com".to_string()),
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Test Browser".to_string()),
        details: Some("Successful login".to_string()),
        success: true,
        created_at: now,
        actor_role: Some("logsmart_admin".to_string()),
        company_id: Some("company1-uuid".to_string()),
        request_method: None,
        request_path: None,
        target_email: None,
        target_user_id: None,
    };

    assert_eq!(log.id, "log1");
    assert_eq!(log.event_type, "login");
    assert_eq!(log.user_id, Some("user1".to_string()));
    assert_eq!(log.email, Some("test@example.com".to_string()));
    assert_eq!(log.success, true);
    assert_eq!(log.created_at, now);
}

#[test]
fn test_security_log_failed_login() {
    let log = SecurityLog {
        success: false,
        event_type: "login_failed".to_string(),
        details: Some("Invalid credentials".to_string()),
        ..create_test_security_log()
    };

    assert!(!log.success);
    assert_eq!(log.event_type, "login_failed");
    assert_eq!(log.details, Some("Invalid credentials".to_string()));
}

#[test]
fn test_security_log_minimal_data() {
    let log = SecurityLog {
        id: "log2".to_string(),
        event_type: "password_reset".to_string(),
        user_id: None,
        email: None,
        ip_address: None,
        user_agent: None,
        details: None,
        success: true,
        created_at: Utc::now(),
        actor_role: None,
        company_id: None,
        request_method: None,
        request_path: None,
        target_email: None,
        target_user_id: None,
    };

    assert!(log.user_id.is_none());
    assert!(log.email.is_none());
    assert!(log.ip_address.is_none());
    assert!(log.user_agent.is_none());
    assert!(log.details.is_none());
    assert!(log.success);
}

#[test]
fn test_passkey_creation() {
    let now = Utc::now();
    let passkey = Passkey {
        id: "pk1".to_string(),
        user_id: "user1".to_string(),
        credential_id: "cred123".to_string(),
        public_key: "public_key_data".to_string(),
        counter: 0,
        name: "My Phone".to_string(),
        created_at: now,
        last_used_at: None,
    };

    assert_eq!(passkey.id, "pk1");
    assert_eq!(passkey.user_id, "user1");
    assert_eq!(passkey.credential_id, "cred123");
    assert_eq!(passkey.counter, 0);
    assert_eq!(passkey.name, "My Phone");
    assert_eq!(passkey.created_at, now);
    assert!(passkey.last_used_at.is_none());
}

#[test]
fn test_passkey_used() {
    let last_used_at = Some(Utc::now());
    let passkey = Passkey {
        last_used_at,
        counter: 5,
        ..create_test_passkey()
    };

    assert!(passkey.last_used_at.is_some());
    assert_eq!(passkey.counter, 5);
}

#[test]
fn test_passkey_session_creation() {
    let now = Utc::now();
    let expires_at = now + Duration::minutes(5);
    let session = PasskeySession {
        id: "session1".to_string(),
        session_type: "registration".to_string(),
        user_id: Some("user1".to_string()),
        challenge: "challenge123".to_string(),
        meta: Some("meta_data".to_string()),
        created_at: now,
        expires_at,
    };

    assert_eq!(session.id, "session1");
    assert_eq!(session.session_type, "registration");
    assert_eq!(session.user_id, Some("user1".to_string()));
    assert_eq!(session.challenge, "challenge123");
    assert_eq!(session.meta, Some("meta_data".to_string()));
    assert_eq!(session.created_at, now);
    assert_eq!(session.expires_at, expires_at);
}

#[test]
fn test_passkey_session_no_user() {
    let session = PasskeySession {
        user_id: None,
        ..create_test_passkey_session()
    };

    assert!(session.user_id.is_none());
}

#[test]
fn test_passkey_session_expired_check() {
    let now = Utc::now();
    let expired_session = PasskeySession {
        expires_at: now - Duration::minutes(1),
        ..create_test_passkey_session()
    };

    let valid_session = PasskeySession {
        expires_at: now + Duration::minutes(5),
        ..create_test_passkey_session()
    };

    assert!(expired_session.expires_at < now);
    assert!(valid_session.expires_at > now);
}

#[test]
fn test_user_display_creation() {
    let display = back_end::db::UserDisplay {
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        company_name: Some("Test Company".to_string()),
        role: "admin".to_string(),
    };

    assert_eq!(display.email, "test@example.com");
    assert_eq!(display.first_name, "Test");
    assert_eq!(display.last_name, "User");
    assert_eq!(display.company_name, Some("Test Company".to_string()));
    assert_eq!(display.role, "admin");
}

#[test]
fn test_user_lifecycle_scenarios() {
    let now = Utc::now();

    let new_user = UserRecord {
        created_at: now,
        deleted_at: None,
        password_hash: Some("hash".to_string()),
        ..create_test_user_record()
    };

    assert!(new_user.password_hash.is_some());
    assert!(new_user.deleted_at.is_none());

    let deleted_user = UserRecord {
        deleted_at: Some(now + Duration::days(1)),
        ..new_user
    };

    assert!(deleted_user.deleted_at.is_some());
}

#[test]
fn test_oauth_user_conversion_flow() {
    let oauth_user = UserRecord {
        password_hash: None,
        oauth_provider: Some("google".to_string()),
        oauth_subject: Some("google_user_123".to_string()),
        oauth_picture: Some("https://example.com/avatar.jpg".to_string()),
        ..create_test_user_record()
    };

    assert!(oauth_user.password_hash.is_none());
    assert_eq!(oauth_user.oauth_provider, Some("google".to_string()));
    assert_eq!(
        oauth_user.oauth_subject,
        Some("google_user_123".to_string())
    );
    assert_eq!(
        oauth_user.oauth_picture,
        Some("https://example.com/avatar.jpg".to_string())
    );
}

#[test]
fn test_invitation_state_transitions() {
    let now = Utc::now();

    let base_invitation = create_test_invitation();
    let invitation = Invitation {
        created_at: now,
        accepted_at: None,
        cancelled_at: None,
        ..base_invitation.clone()
    };

    assert!(invitation.accepted_at.is_none());
    assert!(invitation.cancelled_at.is_none());

    let accepted_invitation = Invitation {
        accepted_at: Some(now + Duration::hours(1)),
        ..base_invitation.clone()
    };

    assert!(accepted_invitation.accepted_at.is_some());

    let cancelled_invitation = Invitation {
        cancelled_at: Some(now + Duration::hours(2)),
        ..base_invitation
    };

    assert!(cancelled_invitation.cancelled_at.is_some());
}

#[test]
fn test_user_email_format_validation() {
    let valid_emails = vec![
        "user@example.com",
        "test.email+tag@domain.co.uk",
        "user_name123@test-domain.com",
        "a@b.co",
    ];

    for email in valid_emails {
        let email_regex =
            regex::Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
        assert!(
            email_regex.is_match(email),
            "Email {} should match regex",
            email
        );
    }
}

#[test]
fn test_user_email_invalid_formats() {
    let invalid_emails = vec![
        "",
        "plainaddress",
        "@missingdomain.com",
        "missing@.com",
        "missing@domain",
        "spaces @domain.com",
        "user@domain .com",
        "user@domain@domain.com",
    ];

    for email in invalid_emails {
        let email_regex =
            regex::Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
        assert!(
            !email_regex.is_match(email),
            "Email {} should not match regex",
            email
        );
    }
}

#[test]
fn test_invitation_email_format_validation() {
    let valid_invitation_emails = vec![
        "invite@example.com",
        "test.invite+tag@domain.co.uk",
        "user_invite123@test-domain.com",
    ];

    for email in valid_invitation_emails {
        let email_regex =
            regex::Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
        assert!(
            email_regex.is_match(email),
            "Invitation email {} should match regex",
            email
        );
    }
}
