use back_end::db::{Company, Invitation, UserRecord, UserRole};
use back_end::logs_db::{
    Frequency, LogEntry, Position, Schedule, TemplateDocument, TemplateField, TemplateFieldProps,
};
use chrono::Utc;
use mongodb::bson::Uuid;

#[test]
fn test_user_creation() {
    let user = UserRecord {
        id: Uuid::new().to_string(),
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        password_hash: Some("hash".to_string()),
        company_id: Some("company123".to_string()),
        company_name: None,
        role: UserRole::Member,
        created_at: Utc::now(),
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    };

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.role, UserRole::Member);
    assert!(user.password_hash.is_some());
}

#[test]
fn test_company_creation() {
    let company = Company {
        id: Uuid::new().to_string(),
        name: "Test Company".to_string(),
        address: "123 Test St".to_string(),
        created_at: Utc::now(),
    };

    assert_eq!(company.name, "Test Company");
    assert_eq!(company.address, "123 Test St");
}

#[test]
fn test_invitation_creation() {
    let invitation = Invitation {
        id: Uuid::new().to_string(),
        email: "invite@example.com".to_string(),
        company_id: "company123".to_string(),
        token: "token123".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        accepted_at: None,
        cancelled_at: None,
    };

    assert_eq!(invitation.email, "invite@example.com");
    assert!(invitation.accepted_at.is_none());
}

#[test]
fn test_template_creation() {
    let template = TemplateDocument {
        template_name: "Test Template".to_string(),
        template_layout: vec![TemplateField {
            field_type: "text".to_string(),
            position: Position { x: 0.0, y: 0.0 },
            props: TemplateFieldProps {
                text: Some("Test Field".to_string()),
                size: None,
                weight: None,
                value: None,
                min: None,
                max: None,
                unit: None,
                selected: None,
                options: None,
                editable: None,
            },
        }],
        company_id: "company123".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        schedule: Schedule {
            frequency: Frequency::Daily,
            days_of_week: None,
            day_of_week: None,
            day_of_month: None,
            month_of_year: None,
        },
        created_by: Uuid::new(),
    };

    assert_eq!(template.template_name, "Test Template");
    assert_eq!(template.template_layout.len(), 1);
}

#[test]
fn test_log_entry_creation() {
    let entry = LogEntry {
        entry_id: Uuid::new().to_string(),
        template_name: "Test Template".to_string(),
        company_id: "company123".to_string(),
        user_id: "user123".to_string(),
        entry_data: serde_json::json!({"field1": "value1"}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        submitted_at: None,
        status: "draft".to_string(),
        period: "2024-01".to_string(),
    };

    assert_eq!(entry.template_name, "Test Template");
    assert_eq!(entry.status, "draft");
    assert!(entry.submitted_at.is_none());
}
