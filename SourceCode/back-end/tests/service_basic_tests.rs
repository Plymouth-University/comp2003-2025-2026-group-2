use back_end::common::factories::*;
use back_end::db::UserRole;

#[tokio::test]
async fn test_user_factory_basic() {
    let user = UserFactory::create_basic();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.first_name, "Test");
    assert_eq!(user.last_name, "User");
    assert_eq!(user.role, UserRole::Staff);
    assert!(user.password_hash.is_some());
}

#[tokio::test]
async fn test_user_factory_company_manager() {
    let user = UserFactory::create_company_manager();
    assert_eq!(user.email, "admin@test.com");
    assert_eq!(user.role, UserRole::CompanyManager);
}

#[tokio::test]
async fn test_company_factory_basic() {
    let company = CompanyFactory::create_basic();
    assert_eq!(company.name, "Test Company");
    assert_eq!(company.address, "123 Test Street");
}

#[tokio::test]
async fn test_invitation_factory_basic() {
    let invitation = InvitationFactory::create_basic();
    assert_eq!(invitation.email, "invite@example.com");
    assert!(!invitation.token.is_empty());
    assert!(invitation.accepted_at.is_none());
    assert!(invitation.cancelled_at.is_none());
}

#[tokio::test]
async fn test_template_factory_basic() {
    let template = TemplateFactory::create_basic();
    assert_eq!(template.template_name, "Test Template");
    assert!(!template.template_layout.is_empty());
}

#[tokio::test]
async fn test_log_entry_factory_basic() {
    let entry = LogEntryFactory::create_basic();
    assert_eq!(entry.template_name, "Test Template");
    assert_eq!(entry.status, "draft");
    assert!(entry.submitted_at.is_none());
}

#[tokio::test]
async fn test_security_log_factory_basic() {
    let log = SecurityLogFactory::create_basic();
    assert_eq!(log.event_type, "login_attempt");
    assert!(log.success);
    assert!(log.user_id.is_some());
}
