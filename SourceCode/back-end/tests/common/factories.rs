//! Test data factories for creating test entities

use crate::db::{Company, UserRecord, UserRole};
use chrono::Utc;
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
            branch_id: None,
            company_name: Some("Test Company".to_string()),
            company_deleted_at: None,
            role: UserRole::Staff,
            created_at: Utc::now(),
            deleted_at: None,
            oauth_provider: None,
            oauth_subject: None,
            oauth_picture: None,
            profile_picture_id: None,
        }
    }

    /// Creates a company manager user
    #[must_use]
    pub fn create_company_manager() -> UserRecord {
        UserRecord {
            role: UserRole::CompanyManager,
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
            ..Company::default()
        }
    }

    /// Creates a deleted company
    #[must_use]
    pub fn create_deleted() -> Company {
        Company {
            deleted_at: Some(Utc::now()),
            ..Self::create_basic()
        }
    }
}
