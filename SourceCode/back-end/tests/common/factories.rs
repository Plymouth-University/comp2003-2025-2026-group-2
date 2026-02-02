//! Test data factories for creating test entities

use crate::{
    db::{Company, Invitation, Passkey, PasskeySession, SecurityLog, UserRecord, UserRole},
    logs_db::{
        Frequency, LogEntry, Position, Schedule, TemplateDocument, TemplateField,
        TemplateFieldProps,
    },
};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

/// Factory for creating test users
pub struct UserFactory;

impl UserFactory {
    /// Creates a basic test user with default values
    #[must_use]
    pub fn create_basic() -> UserRecord {
        UserRecord {
            id: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password_hash: Some("hashed_password".to_string()),
            company_id: Some(Uuid::new_v4().to_string()),
            company_name: Some("Test Company".to_string()),
            role: UserRole::Member,
            created_at: Utc::now(),
            deleted_at: None,
            oauth_provider: None,
            oauth_subject: None,
            oauth_picture: None,
        }
    }

    /// Creates an admin user
    #[must_use]
    pub fn create_admin() -> UserRecord {
        UserRecord {
            role: UserRole::Admin,
            email: "admin@test.com".to_string(),
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
            ..Self::create_basic()
        }
    }

    /// Creates a `LogSmart` admin user
    #[must_use]
    pub fn create_logsmart_admin() -> UserRecord {
        UserRecord {
            role: UserRole::LogSmartAdmin,
            email: "logsmart@logsmart.com".to_string(),
            first_name: "LogSmart".to_string(),
            last_name: "Admin".to_string(),
            ..Self::create_basic()
        }
    }

    /// Creates a user with OAuth provider
    #[must_use]
    pub fn create_oauth_user(provider: &str, subject: &str) -> UserRecord {
        UserRecord {
            oauth_provider: Some(provider.to_string()),
            oauth_subject: Some(subject.to_string()),
            password_hash: None, // OAuth users don't have passwords
            email: "oauth@example.com".to_string(),
            ..Self::create_basic()
        }
    }

    /// Creates a soft-deleted user
    #[must_use]
    pub fn create_deleted() -> UserRecord {
        UserRecord {
            deleted_at: Some(Utc::now()),
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test companies
pub struct CompanyFactory;

impl CompanyFactory {
    /// Creates a basic test company
    #[must_use]
    pub fn create_basic() -> Company {
        Company {
            id: Uuid::new_v4().to_string(),
            name: "Test Company".to_string(),
            address: "123 Test Street".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Creates a company with a specific name
    #[must_use]
    pub fn create_with_name(name: &str) -> Company {
        Company {
            name: name.to_string(),
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test invitations
pub struct InvitationFactory;

impl InvitationFactory {
    /// Creates a basic test invitation
    #[must_use]
    pub fn create_basic() -> Invitation {
        Invitation {
            id: Uuid::new_v4().to_string(),
            email: "invite@example.com".to_string(),
            company_id: Uuid::new_v4().to_string(),
            token: Uuid::new_v4().to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            created_at: Utc::now(),
            accepted_at: None,
            cancelled_at: None,
        }
    }

    /// Creates an expired invitation
    #[must_use]
    pub fn create_expired() -> Invitation {
        Invitation {
            expires_at: Utc::now() - chrono::Duration::hours(1),
            ..Self::create_basic()
        }
    }

    /// Creates an accepted invitation
    #[must_use]
    pub fn create_accepted() -> Invitation {
        Invitation {
            accepted_at: Some(Utc::now()),
            ..Self::create_basic()
        }
    }

    /// Creates a cancelled invitation
    #[must_use]
    pub fn create_cancelled() -> Invitation {
        Invitation {
            cancelled_at: Some(Utc::now()),
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test passkeys
pub struct PasskeyFactory;

impl PasskeyFactory {
    /// Creates a basic test passkey
    #[must_use]
    pub fn create_basic() -> Passkey {
        Passkey {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            credential_id: "test_credential_id".to_string(),
            public_key: "[1, 2, 3, 4, 5]".to_string(), // JSON serialized public key
            counter: 0,
            created_at: Utc::now(),
            last_used_at: None,
            name: "Test Passkey".to_string(),
        }
    }

    /// Creates a passkey that has been used
    #[must_use]
    pub fn create_used() -> Passkey {
        Passkey {
            last_used_at: Some(Utc::now()),
            counter: 1,
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test passkey sessions
pub struct PasskeySessionFactory;

impl PasskeySessionFactory {
    /// Creates a basic passkey session
    #[must_use]
    pub fn create_basic() -> PasskeySession {
        PasskeySession {
            id: Uuid::new_v4().to_string(),
            session_type: "registration".to_string(),
            user_id: Some(Uuid::new_v4().to_string()),
            challenge: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            meta: None,
        }
    }
}

/// Factory for creating test security logs
pub struct SecurityLogFactory;

impl SecurityLogFactory {
    /// Creates a basic security log entry
    #[must_use]
    pub fn create_basic() -> SecurityLog {
        SecurityLog {
            id: Uuid::new_v4().to_string(),
            event_type: "login_attempt".to_string(),
            user_id: Some(Uuid::new_v4().to_string()),
            email: Some("test@example.com".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Test Browser".to_string()),
            details: Some("Successful login".to_string()),
            success: true,
            created_at: Utc::now(),
        }
    }

    /// Creates a failed login attempt log
    #[must_use]
    pub fn create_failed_login() -> SecurityLog {
        SecurityLog {
            event_type: "login_failed".to_string(),
            success: false,
            details: Some("Invalid credentials".to_string()),
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test template documents
pub struct TemplateFactory;

impl TemplateFactory {
    /// Creates a basic test template
    #[must_use]
    pub fn create_basic() -> TemplateDocument {
        TemplateDocument {
            template_name: "Test Template".to_string(),
            template_layout: vec![TemplateField {
                field_type: "text".to_string(),
                position: Position { x: 0.0, y: 0.0 },
                props: TemplateFieldProps {
                    text: Some("Test Field".to_string()),
                    size: Some("medium".to_string()),
                    weight: Some("normal".to_string()),
                    value: None,
                    min: None,
                    max: None,
                    unit: None,
                    selected: None,
                    options: None,
                    editable: Some(true),
                },
            }],
            company_id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schedule: Schedule {
                frequency: Frequency::Daily,
                days_of_week: None,
                day_of_week: None,
                day_of_month: None,
                month_of_year: None,
            },
            created_by: mongodb::bson::Uuid::new(),
        }
    }

    /// Creates a template with schedule
    #[must_use]
    pub fn create_with_schedule() -> TemplateDocument {
        TemplateDocument {
            schedule: Schedule {
                frequency: Frequency::Weekly,
                days_of_week: Some(vec![1, 2, 3, 4, 5]), // Monday to Friday
                day_of_week: Some(1),                    // Monday
                day_of_month: None,
                month_of_year: None,
            },
            ..Self::create_basic()
        }
    }
}

/// Factory for creating test log entry documents
pub struct LogEntryFactory;

impl LogEntryFactory {
    /// Creates a basic log entry
    #[must_use]
    pub fn create_basic() -> LogEntry {
        LogEntry {
            entry_id: mongodb::bson::Uuid::new().to_string(),
            template_name: "Test Template".to_string(),
            company_id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            entry_data: serde_json::json!({
                "field1": "Test value",
                "field2": 123
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            submitted_at: None,
            status: "draft".to_string(),
            period: "2024-01".to_string(),
        }
    }

    /// Creates a submitted log entry
    #[must_use]
    pub fn create_submitted() -> LogEntry {
        LogEntry {
            submitted_at: Some(Utc::now()),
            status: "submitted".to_string(),
            ..Self::create_basic()
        }
    }
}

/// Helper trait for creating test data with custom overrides
pub trait TestFactory<T> {
    fn create() -> T;
    fn with_overrides<F>(self, overrides: F) -> T
    where
        F: FnOnce(&mut T);
}

// Implement TestFactory for all factories
impl TestFactory<UserRecord> for UserFactory {
    fn create() -> UserRecord {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> UserRecord
    where
        F: FnOnce(&mut UserRecord),
    {
        let mut user = Self::create_basic();
        overrides(&mut user);
        user
    }
}

impl TestFactory<Company> for CompanyFactory {
    fn create() -> Company {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> Company
    where
        F: FnOnce(&mut Company),
    {
        let mut company = Self::create_basic();
        overrides(&mut company);
        company
    }
}

impl TestFactory<Invitation> for InvitationFactory {
    fn create() -> Invitation {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> Invitation
    where
        F: FnOnce(&mut Invitation),
    {
        let mut invitation = Self::create_basic();
        overrides(&mut invitation);
        invitation
    }
}

impl TestFactory<Passkey> for PasskeyFactory {
    fn create() -> Passkey {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> Passkey
    where
        F: FnOnce(&mut Passkey),
    {
        let mut passkey = Self::create_basic();
        overrides(&mut passkey);
        passkey
    }
}

impl TestFactory<PasskeySession> for PasskeySessionFactory {
    fn create() -> PasskeySession {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> PasskeySession
    where
        F: FnOnce(&mut PasskeySession),
    {
        let mut session = Self::create_basic();
        overrides(&mut session);
        session
    }
}

impl TestFactory<SecurityLog> for SecurityLogFactory {
    fn create() -> SecurityLog {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> SecurityLog
    where
        F: FnOnce(&mut SecurityLog),
    {
        let mut log = Self::create_basic();
        overrides(&mut log);
        log
    }
}

impl TestFactory<TemplateDocument> for TemplateFactory {
    fn create() -> TemplateDocument {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> TemplateDocument
    where
        F: FnOnce(&mut TemplateDocument),
    {
        let mut template = Self::create_basic();
        overrides(&mut template);
        template
    }
}

impl TestFactory<LogEntry> for LogEntryFactory {
    fn create() -> LogEntry {
        Self::create_basic()
    }

    fn with_overrides<F>(self, overrides: F) -> LogEntry
    where
        F: FnOnce(&mut LogEntry),
    {
        let mut entry = Self::create_basic();
        overrides(&mut entry);
        entry
    }
}

// Database creation helper functions for tests

/// Creates a test user in the database.
///
/// # Panics
/// Panics if database operations fail.
pub async fn create_test_user(
    pool: &sqlx::PgPool,
    email: &str,
    company_id: Option<&str>,
) -> UserRecord {
    let mut user = UserFactory::create_basic();
    user.email = email.to_string();
    if let Some(cid) = company_id {
        user.company_id = Some(cid.to_string());
    }

    let role_clone = user.role.clone();
    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role)
        VALUES ($1, $2, $3, $4, $5, $6, $7::user_role)
        RETURNING id, email, first_name, last_name, password_hash, company_id, created_at, deleted_at, oauth_provider, oauth_subject, oauth_picture
        "
    )
    .bind(user.id)
    .bind(user.email)
    .bind(user.first_name)
    .bind(user.last_name)
    .bind(user.password_hash)
    .bind(user.company_id)
    .bind(role_clone.to_string())
    .fetch_one(pool)
    .await
    .map(|row| UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        password_hash: row.get("password_hash"),
        company_id: row.get("company_id"),
        company_name: None,
        role: role_clone,
        created_at: row.get("created_at"),
        deleted_at: row.get("deleted_at"),
        oauth_provider: row.get("oauth_provider"),
        oauth_subject: row.get("oauth_subject"),
        oauth_picture: row.get("oauth_picture"),
    })
    .unwrap()
}

/// Creates a test user with a specific role.
///
/// # Panics
/// Panics if database operations fail.
pub async fn create_test_user_with_role(
    pool: &sqlx::PgPool,
    email: &str,
    role: UserRole,
    company_id: Option<&str>,
) -> UserRecord {
    let mut user = UserFactory::create_basic();
    user.email = email.to_string();
    user.role = role;
    if let Some(cid) = company_id {
        user.company_id = Some(cid.to_string());
    }

    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role)
        VALUES ($1, $2, $3, $4, $5, $6, $7::user_role)
        RETURNING id, email, first_name, last_name, password_hash, company_id, role, created_at, deleted_at, oauth_provider, oauth_subject, oauth_picture
        "
    )
    .bind(user.id)
    .bind(user.email)
    .bind(user.first_name)
    .bind(user.last_name)
    .bind(user.password_hash)
    .bind(user.company_id)
    .bind(user.role.to_string())
    .fetch_one(pool)
    .await
    .map(|row| UserRecord {
        id: row.get("id"),
        email: row.get("email"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        password_hash: row.get("password_hash"),
        company_id: row.get("company_id"),
        company_name: None,
        role: user.role,
        created_at: row.get("created_at"),
        deleted_at: row.get("deleted_at"),
        oauth_provider: row.get("oauth_provider"),
        oauth_subject: row.get("oauth_subject"),
        oauth_picture: row.get("oauth_picture"),
    })
    .unwrap()
}

/// Creates a test company in the database.
///
/// # Panics
/// Panics if database operations fail.
pub async fn create_test_company(pool: &sqlx::PgPool, name: &str, address: &str) -> Company {
    let mut company = CompanyFactory::create_basic();
    company.name = name.to_string();
    company.address = address.to_string();

    sqlx::query!(
        "INSERT INTO companies (id, name, address) VALUES ($1, $2, $3) RETURNING id, name, address, created_at",
        company.id,
        company.name,
        company.address
    )
    .fetch_one(pool)
    .await
    .map(|row| Company {
        id: row.id,
        name: row.name,
        address: row.address,
        created_at: row.created_at.unwrap(),
    })
    .unwrap()
}

/// Creates a test invitation in the database.
///
/// # Panics
/// Panics if database operations fail.
pub async fn create_test_invitation(
    pool: &sqlx::PgPool,
    company_id: &str,
    email: &str,
) -> Invitation {
    let mut invitation = InvitationFactory::create_basic();
    invitation.company_id = company_id.to_string();
    invitation.email = email.to_string();

    sqlx::query!(
        r"
        INSERT INTO invitations (id, company_id, email, token, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
        ",
        invitation.id,
        invitation.company_id,
        invitation.email,
        invitation.token,
        invitation.expires_at,
        invitation.created_at
    )
    .fetch_one(pool)
    .await
    .map(|row| Invitation {
        id: row.id,
        company_id: row.company_id,
        email: row.email,
        token: row.token,
        created_at: row.created_at.unwrap(),
        expires_at: row.expires_at,
        accepted_at: row.accepted_at,
        cancelled_at: row.cancelled_at,
    })
    .unwrap()
}

/// Creates a test invitation with specific expiry.
///
/// # Panics
/// Panics if database operations fail.
pub async fn create_test_invitation_with_expiry(
    pool: &sqlx::PgPool,
    company_id: &str,
    email: &str,
    expires_at: DateTime<Utc>,
) -> Invitation {
    let mut invitation = InvitationFactory::create_basic();
    invitation.company_id = company_id.to_string();
    invitation.email = email.to_string();
    invitation.expires_at = expires_at;

    sqlx::query!(
        r"
        INSERT INTO invitations (id, company_id, email, token, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
        ",
        invitation.id,
        invitation.company_id,
        invitation.email,
        invitation.token,
        invitation.expires_at,
        invitation.created_at
    )
    .fetch_one(pool)
    .await
    .map(|row| Invitation {
        id: row.id,
        company_id: row.company_id,
        email: row.email,
        token: row.token,
        created_at: row.created_at.unwrap(),
        expires_at: row.expires_at,
        accepted_at: row.accepted_at,
        cancelled_at: row.cancelled_at,
    })
    .unwrap()
}
