use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt::Write;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "logsmart_admin")]
    #[sqlx(rename = "logsmart_admin")]
    LogSmartAdmin,
    #[serde(rename = "company_manager")]
    #[sqlx(rename = "company_manager")]
    CompanyManager,
    #[serde(rename = "branch_manager")]
    #[sqlx(rename = "branch_manager")]
    BranchManager,
    #[serde(rename = "staff")]
    #[sqlx(rename = "staff")]
    Staff,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::LogSmartAdmin => write!(f, "logsmart_admin"),
            UserRole::CompanyManager => write!(f, "company_manager"),
            UserRole::BranchManager => write!(f, "branch_manager"),
            UserRole::Staff => write!(f, "staff"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "logsmart_admin" => Ok(UserRole::LogSmartAdmin),
            "company_manager" => Ok(UserRole::CompanyManager),
            "branch_manager" => Ok(UserRole::BranchManager),
            "staff" => Ok(UserRole::Staff),
            _ => Err(format!("Unknown role: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDisplay {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: Option<String>,
    pub company_id: Option<String>,
    pub branch_id: Option<String>,
    pub company_name: Option<String>,
    pub company_deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub oauth_picture: Option<String>,
    pub profile_picture_id: Option<String>,
}

impl UserRecord {
    #[must_use]
    pub fn get_role(&self) -> UserRole {
        self.role.clone()
    }

    #[must_use]
    pub fn is_logsmart_admin(&self) -> bool {
        self.get_role() == UserRole::LogSmartAdmin
    }

    #[must_use]
    pub fn is_company_manager(&self) -> bool {
        self.get_role() == UserRole::CompanyManager
    }

    #[must_use]
    pub fn is_branch_manager(&self) -> bool {
        self.get_role() == UserRole::BranchManager
    }

    #[must_use]
    pub fn is_staff(&self) -> bool {
        self.get_role() == UserRole::Staff
    }

    #[must_use]
    pub fn can_manage_company(&self) -> bool {
        self.is_company_manager() || self.is_logsmart_admin()
    }

    #[must_use]
    pub fn can_manage_branch(&self) -> bool {
        self.is_branch_manager() || self.can_manage_company()
    }

    #[must_use]
    pub fn is_readonly_hq(&self) -> bool {
        self.is_staff() && self.branch_id.is_none()
    }

    #[must_use]
    pub fn can_read_manage_branch(&self) -> bool {
        self.is_readonly_hq() || self.can_manage_branch()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq)]
pub struct Branch {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub logo_id: Option<String>,
    pub data_exported_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_token: Option<String>,
    pub deletion_requested_by_email: Option<String>,
}

impl Default for Company {
    fn default() -> Self {
        Self::new()
    }
}

impl Company {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            address: String::new(),
            created_at: chrono::Utc::now(),
            logo_id: None,
            data_exported_at: None,
            deleted_at: None,
            deletion_requested_at: None,
            deletion_token: None,
            deletion_requested_by_email: None,
        }
    }

    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn with_name_and_address(mut self, name: &str, address: &str) -> Self {
        self.name = name.to_string();
        self.address = address.to_string();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invitation {
    pub id: String,
    pub company_id: String,
    pub email: String,
    pub token: String,
    pub role: UserRole,
    pub branch_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecurityLog {
    pub id: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub success: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Passkey {
    pub id: String,
    pub user_id: String,
    pub credential_id: String, // Base64URL encoded
    pub public_key: String,    // Json serialized
    pub counter: i64,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasskeySession {
    pub id: String,
    pub session_type: String,
    pub user_id: Option<String>,
    pub challenge: String,
    pub meta: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClockEvent {
    pub id: String,
    pub user_id: String,
    pub company_id: String,
    pub clock_in: chrono::DateTime<chrono::Utc>,
    pub clock_out: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ClockEvent {
    #[must_use]
    pub fn is_clocked_in(&self) -> bool {
        self.clock_out.is_none()
    }
}

/// Initialize database by running `SQLx` migrations
///
/// This function runs all pending migrations from the `migrations/` directory.
/// Migrations are applied in order based on their timestamp prefix.
///
/// # Errors
/// Returns an error if migrations fail to execute.
pub async fn init_db(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {e}"))?;

    Ok(())
}

/// Creates a new user in the database.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_user<'a, E>(
    executor: E,
    email: String,
    first_name: String,
    last_name: String,
    password_hash: Option<String>,
    company_id: Option<String>,
    role: UserRole,
) -> Result<UserRecord>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "
    )
    .bind(&id)
    .bind(&email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&password_hash)
    .bind(&company_id)
    .bind(&role)
    .bind(now)
    .execute(executor)
    .await?;

    Ok(UserRecord {
        id,
        email,
        first_name,
        last_name,
        password_hash,
        company_id,
        branch_id: None,
        company_name: None,
        profile_picture_id: None,
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
        company_deleted_at: None,
    })
}

/// Retrieves a user's company ID by their user ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_user_company_id(pool: &PgPool, user_id: &str) -> Result<Option<String>> {
    #[derive(sqlx::FromRow)]
    struct CompanyIdRow {
        company_id: Option<String>,
    }

    let record = sqlx::query_as::<_, CompanyIdRow>(
        r"
        SELECT company_id
        FROM users
        WHERE id = $1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(record.and_then(|r| r.company_id))
}

/// Retrieves a user by their email address.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
               companies.name as company_name, companies.deleted_at as company_deleted_at,
               users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.email = $1 AND users.deleted_at IS NULL
        ",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Retrieves a user by their role.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_users_by_role(
    pool: &PgPool,
    id: &str,
    company_id: &str,
    role: &str,
    branch_id: Option<String>,
) -> Result<Vec<UserRecord>> {
    let users = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
               companies.name as company_name, companies.deleted_at as company_deleted_at,
               users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE NOT users.id = $1 AND users.company_id = $2 AND users.role = $3 AND users.branch_id = $4 AND users.deleted_at IS NULL
        ",
    )
    .bind(id)
    .bind(company_id)
    .bind(role)
    .bind(&branch_id)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Retrieves a user by their ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_user_by_id(pool: &PgPool, id: &str) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
            users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
            companies.name as company_name, companies.deleted_at as company_deleted_at,
            users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.id = $1 AND users.deleted_at IS NULL
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if let Some(u) = &user {
        tracing::debug!(
            "DB: Found user {} with company_id: {:?}, branch_id: {:?}",
            u.email,
            u.company_id,
            u.branch_id
        );
    } else {
        tracing::debug!("DB: User {} not found", id);
    }

    Ok(user)
}

/// Retrieves a user by OAuth provider and subject.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_user_by_oauth(
    pool: &PgPool,
    provider: &str,
    subject: &str,
) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
            users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
            companies.name as company_name, companies.deleted_at as company_deleted_at,
            users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.oauth_provider = $1 AND users.oauth_subject = $2 AND users.deleted_at IS NULL
        ",
    )
    .bind(provider)
    .bind(subject)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Creates a new user with OAuth authentication.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_oauth_user<'a, E>(
    executor: E,
    email: String,
    first_name: String,
    last_name: String,
    oauth_provider: String,
    oauth_subject: String,
    oauth_picture: Option<String>,
    company_id: Option<String>,
    role: UserRole,
) -> Result<UserRecord>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at, oauth_provider, oauth_subject, oauth_picture)
        VALUES ($1, $2, $3, $4, NULL, $5, $6, $7, $8, $9, $10)
        "
    )
    .bind(&id)
    .bind(&email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&company_id)
    .bind(&role)
    .bind(now)
    .bind(&oauth_provider)
    .bind(&oauth_subject)
    .bind(&oauth_picture)
    .execute(executor)
    .await?;

    Ok(UserRecord {
        id,
        email,
        first_name,
        last_name,
        password_hash: None,
        company_id,
        branch_id: None,
        company_name: None,
        role,
        created_at: now,
        deleted_at: None,
        profile_picture_id: None,
        oauth_provider: Some(oauth_provider),
        oauth_subject: Some(oauth_subject),
        oauth_picture,
        company_deleted_at: None,
    })
}

/// Links an OAuth account to an existing user.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn link_oauth_to_user(
    pool: &PgPool,
    user_id: &str,
    oauth_provider: String,
    oauth_subject: String,
    oauth_picture: Option<String>,
) -> Result<()> {
    sqlx::query(
        r"
        UPDATE users
        SET oauth_provider = $1, oauth_subject = $2, oauth_picture = $3
        WHERE id = $4
        ",
    )
    .bind(&oauth_provider)
    .bind(&oauth_subject)
    .bind(&oauth_picture)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Unlinks OAuth authentication from a user.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn unlink_oauth_from_user(pool: &PgPool, user_id: &str) -> Result<()> {
    sqlx::query(
        r"
        UPDATE users
        SET oauth_provider = NULL, oauth_subject = NULL, oauth_picture = NULL
        WHERE id = $1
        ",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Soft deletes a user by their email address.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn delete_user_by_email(pool: &PgPool, email: &str) -> Result<()> {
    sqlx::query(
        r"
        UPDATE users
        SET deleted_at = $1
        WHERE email = $2 AND deleted_at IS NULL
        ",
    )
    .bind(chrono::Utc::now())
    .bind(email)
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a new company in the database.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_company<'a, E>(executor: E, name: String, address: String) -> Result<Company>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO companies (id, name, address, created_at)
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(&id)
    .bind(&name)
    .bind(&address)
    .bind(now)
    .execute(executor)
    .await?;

    Ok(Company {
        id,
        name,
        address,
        created_at: now,
        ..Company::new()
    })
}

/// Updates a given company's logo
///
/// # Errors
/// Returns an error if database query fails
pub async fn update_company_logo_id(
    pool: &PgPool,
    company_id: &str,
    company_logo_id: Option<&str>,
) -> Result<Company> {
    sqlx::query_as(
        r"
        UPDATE companies
        SET logo_id = $1
        WHERE id = $2
        RETURNING id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        ",
    )
    .bind(company_logo_id)
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to update company logo: {}", e))
}

/// Retrieves a company by its ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_company_by_id(pool: &PgPool, id: &str) -> Result<Option<Company>> {
    let company = sqlx::query_as::<_, Company>(
        r"
        SELECT id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        FROM companies
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(company)
}

/// Updates a company's name and address.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn update_company(
    pool: &PgPool,
    company_id: &str,
    name: &str,
    address: &str,
) -> Result<Company> {
    sqlx::query_as(
        r"
        UPDATE companies
        SET name = $1, address = $2
        WHERE id = $3
        RETURNING id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        ",
    )
    .bind(name)
    .bind(address)
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to update company: {}", e))
}

/// Marks company data as exported.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn mark_company_data_exported(pool: &PgPool, company_id: &str) -> Result<Company> {
    sqlx::query_as(
        r"
        UPDATE companies
        SET data_exported_at = NOW()
        WHERE id = $1
        RETURNING id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        ",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to mark company data as exported: {}", e))
}

/// Requests company deletion with a confirmation token.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn request_company_deletion(
    pool: &PgPool,
    company_id: &str,
    requester_email: &str,
) -> Result<Company> {
    let token = Uuid::new_v4().to_string();
    sqlx::query_as(
        r"
        UPDATE companies
        SET deletion_requested_at = NOW(), deletion_token = $1, deletion_requested_by_email = $2
        WHERE id = $3
        RETURNING id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        ",
    )
    .bind(&token)
    .bind(requester_email)
    .bind(company_id)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to request company deletion: {}", e))
}

/// Confirms company deletion with token.
///
/// # Errors
/// Returns an error if database query fails or token is invalid.
pub async fn confirm_company_deletion(
    pool: &PgPool,
    company_id: &str,
    token: &str,
) -> Result<Company> {
    let company = sqlx::query_as(
        r"
        UPDATE companies
        SET deleted_at = NOW(), deletion_token = NULL
        WHERE id = $1 AND deletion_token = $2 AND deletion_requested_at IS NOT NULL
        RETURNING id, name, address, created_at, logo_id, data_exported_at, deleted_at, deletion_requested_at, deletion_token, deletion_requested_by_email
        ",
    )
    .bind(company_id)
    .bind(token)
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to confirm company deletion: {}", e))?;

    Ok(company)
}

/// Creates a new invitation for a user to join a company.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_invitation(
    pool: &PgPool,
    company_id: String,
    email: String,
    token: String,
    role: UserRole,
    branch_id: Option<String>,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<Invitation> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO invitations (id, company_id, email, token, role, branch_id, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
    )
    .bind(&id)
    .bind(&company_id)
    .bind(&email)
    .bind(&token)
    .bind(&role)
    .bind(&branch_id)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(Invitation {
        id,
        company_id,
        email,
        token,
        role,
        branch_id,
        created_at: now,
        expires_at,
        accepted_at: None,
        cancelled_at: None,
    })
}

/// Retrieves an invitation by its token.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_invitation_by_token(pool: &PgPool, token: &str) -> Result<Option<Invitation>> {
    let invitation = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, role, branch_id, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE token = $1 AND accepted_at IS NULL AND cancelled_at IS NULL
        ",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

/// Marks an invitation as accepted.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn accept_invitation(pool: &PgPool, invitation_id: &str) -> Result<()> {
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        UPDATE invitations
        SET accepted_at = $1
        WHERE id = $2
        ",
    )
    .bind(now)
    .bind(invitation_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Cancels a pending invitation.
///
/// # Errors
/// Returns an error if database update fails or invitation not found.
pub async fn cancel_invitation(pool: &PgPool, invitation_id: &str) -> Result<Invitation> {
    let now = chrono::Utc::now();

    let invitation = sqlx::query_as::<_, Invitation>(
        r"
        UPDATE invitations
        SET cancelled_at = $1
        WHERE id = $2 AND accepted_at IS NULL AND cancelled_at IS NULL
        RETURNING id, company_id, email, token, role, branch_id, created_at, expires_at, accepted_at, cancelled_at
        ",
    )
    .bind(now)
    .bind(invitation_id)
    .fetch_one(pool)
    .await?;

    Ok(invitation)
}

/// Retrieves an invitation by its ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_invitation_by_id(
    pool: &PgPool,
    invitation_id: &str,
) -> Result<Option<Invitation>> {
    let invitation = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, role, branch_id, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE id = $1
        ",
    )
    .bind(invitation_id)
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

/// Logs a security event to the database.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn log_security_event(
    pool: &PgPool,
    event_type: String,
    user_id: Option<String>,
    email: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    details: Option<String>,
    success: bool,
) -> Result<SecurityLog> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO security_logs (id, event_type, user_id, email, ip_address, user_agent, details, success, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "
    )
    .bind(&id)
    .bind(&event_type)
    .bind(&user_id)
    .bind(&email)
    .bind(&ip_address)
    .bind(&user_agent)
    .bind(&details)
    .bind(success)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(SecurityLog {
        id,
        event_type,
        user_id,
        email,
        ip_address,
        user_agent,
        details,
        success,
        created_at: now,
    })
}

/// Retrieves security logs for a specific user.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_security_logs_by_user(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<SecurityLog>> {
    let logs = sqlx::query_as::<_, SecurityLog>(
        r"
        SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
        FROM security_logs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        ",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

/// Retrieves recent security logs, optionally filtered by event type.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_recent_security_logs(
    pool: &PgPool,
    event_type: Option<String>,
    limit: i64,
) -> Result<Vec<SecurityLog>> {
    let logs = if let Some(evt) = event_type {
        sqlx::query_as::<_, SecurityLog>(
            r"
            SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
            FROM security_logs
            WHERE event_type = $1
            ORDER BY created_at DESC
            LIMIT $2
            ",
        )
        .bind(evt)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, SecurityLog>(
            r"
            SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
            FROM security_logs
            ORDER BY created_at DESC
            LIMIT $1
            ",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(logs)
}

/// Updates a user's profile information (name only).
///
/// # Errors
/// Returns an error if database update fails or user not found.
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: &str,
    first_name: String,
    last_name: String,
) -> Result<UserRecord> {
    sqlx::query(
        r"
        UPDATE users
        SET first_name = $1, last_name = $2
        WHERE id = $3
        ",
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(user_id)
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(user)
}

pub async fn update_user_profile_picture_id(
    pool: &PgPool,
    user_id: &str,
    picture_id: Option<&str>,
) -> Result<UserRecord> {
    sqlx::query(
        r"
        UPDATE users
        SET profile_picture_id = $1
        WHERE id = $2
        ",
    )
    .bind(picture_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(user)
}

/// Updates a user's profile information including their role and branch.
///
/// # Errors
/// Returns an error if database update fails or user not found.
pub async fn update_user_profile_full(
    pool: &PgPool,
    user_id: &str,
    first_name: String,
    last_name: String,
    role: UserRole,
    branch_id: Option<String>,
    profile_picture_id: Option<String>,
) -> Result<UserRecord> {
    sqlx::query(
        r"
        UPDATE users
        SET first_name = $1, last_name = $2, role = $3, branch_id = $4, profile_picture_id = $5
        WHERE id = $6
        ",
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(&role)
    .bind(&branch_id)
    .bind(&profile_picture_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(user)
}

/// Updates a user's password hash.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn update_user_password(
    pool: &PgPool,
    user_id: &str,
    password_hash: String,
) -> Result<()> {
    sqlx::query(
        r"
        UPDATE users
        SET password_hash = $1
        WHERE id = $2
        ",
    )
    .bind(&password_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a new passkey for a user.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_passkey(
    pool: &PgPool,
    user_id: &str,
    credential_id: String,
    public_key: String,
    name: String,
) -> Result<Passkey> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO passkeys (id, user_id, credential_id, public_key, counter, name, created_at)
        VALUES ($1, $2, $3, $4, 0, $5, $6)
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&credential_id)
    .bind(&public_key)
    .bind(&name)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Passkey {
        id,
        user_id: user_id.to_string(),
        credential_id,
        public_key,
        counter: 0,
        name,
        created_at: now,
        last_used_at: None,
    })
}

/// Retrieves all passkeys for a specific user.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_passkeys_by_user(pool: &PgPool, user_id: &str) -> Result<Vec<Passkey>> {
    let passkeys = sqlx::query_as::<_, Passkey>(
        r"
        SELECT id, user_id, credential_id, public_key, counter, name, created_at, last_used_at
        FROM passkeys
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(passkeys)
}

/// Retrieves a passkey by its credential ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_passkey_by_credential_id(
    pool: &PgPool,
    credential_id: &str,
) -> Result<Option<Passkey>> {
    let passkey = sqlx::query_as::<_, Passkey>(
        r"
        SELECT id, user_id, credential_id, public_key, counter, name, created_at, last_used_at
        FROM passkeys
        WHERE credential_id = $1
        ",
    )
    .bind(credential_id)
    .fetch_optional(pool)
    .await?;

    Ok(passkey)
}

/// Updates a passkey's usage counter and last used timestamp.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn update_passkey_usage(pool: &PgPool, id: &str, counter: i64) -> Result<()> {
    sqlx::query(
        r"
        UPDATE passkeys
        SET counter = $1, last_used_at = $2
        WHERE id = $3
        ",
    )
    .bind(counter)
    .bind(chrono::Utc::now())
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Deletes a passkey for a specific user.
///
/// # Errors
/// Returns an error if database deletion fails.
pub async fn delete_passkey(pool: &PgPool, id: &str, user_id: &str) -> Result<()> {
    sqlx::query(
        r"
        DELETE FROM passkeys
        WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a password reset token for a user.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_password_reset_token(
    pool: &PgPool,
    user_id: String,
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO password_resets (id, user_id, token, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&token)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Retrieves a password reset record by its token.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_password_reset_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<(String, String)>> {
    let result = sqlx::query_as::<_, (String, String)>(
        r"
        SELECT id, user_id
        FROM password_resets
        WHERE token = $1 AND used_at IS NULL AND expires_at > now()
        ",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// Marks a password reset token as used.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn mark_password_reset_used(pool: &PgPool, reset_id: &str) -> Result<()> {
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        UPDATE password_resets
        SET used_at = $1
        WHERE id = $2
        ",
    )
    .bind(now)
    .bind(reset_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves all users belonging to a specific company.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_users_by_company_id(pool: &PgPool, company_id: &str) -> Result<Vec<UserRecord>> {
    let users = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
               companies.name as company_name, companies.deleted_at as company_deleted_at,
               users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.company_id = $1 AND users.deleted_at IS NULL
        "
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Retrieves all users belonging to a specific company, including soft-deleted ones.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_all_users_by_company_id(
    pool: &PgPool,
    company_id: &str,
) -> Result<Vec<UserRecord>> {
    let users = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.branch_id, users.role, users.created_at, users.deleted_at, 
               companies.name as company_name, companies.deleted_at as company_deleted_at,
               users.oauth_provider, users.oauth_subject, users.oauth_picture, users.profile_picture_id
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.company_id = $1
        "
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Updates the company association for a user.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn update_user_company<'a, E>(executor: E, user_id: &str, company_id: &str) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query(
        r"
        UPDATE users
        SET company_id = $1
        WHERE id = $2
        ",
    )
    .bind(company_id)
    .bind(user_id)
    .execute(executor)
    .await?;

    Ok(())
}

/// Creates a new branch for a company.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_branch<'a, E>(
    executor: E,
    company_id: String,
    name: String,
    address: String,
) -> Result<Branch>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO branches (id, company_id, name, address, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(&id)
    .bind(&company_id)
    .bind(&name)
    .bind(&address)
    .bind(now)
    .execute(executor)
    .await?;

    Ok(Branch {
        id,
        company_id,
        name,
        address,
        created_at: now,
    })
}

/// Retrieves all branches for a company.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_branches_by_company_id(pool: &PgPool, company_id: &str) -> Result<Vec<Branch>> {
    let branches = sqlx::query_as::<_, Branch>(
        r"
        SELECT id, company_id, name, address, created_at
        FROM branches
        WHERE company_id = $1
        ORDER BY name ASC
        ",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(branches)
}

/// Branch with deletion status information
pub struct BranchWithDeletionStatus {
    pub branch: Branch,
    pub has_pending_deletion: bool,
    pub deletion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Retrieves all branches for a company with their deletion status.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_branches_by_company_id_with_deletion_status(
    pool: &PgPool,
    company_id: &str,
) -> Result<Vec<BranchWithDeletionStatus>> {
    let branches = sqlx::query_as::<_, Branch>(
        r"
        SELECT id, company_id, name, address, created_at
        FROM branches
        WHERE company_id = $1
        ORDER BY name ASC
        ",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for branch in branches {
        let deletion_info: Option<(bool, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
            r"
            SELECT 
                CASE WHEN COUNT(*) > 0 THEN true ELSE false END as has_pending,
                MIN(created_at) as requested_at
            FROM branch_deletion_tokens
            WHERE branch_id = $1 AND used_at IS NULL AND expires_at > now()
            ",
        )
        .bind(&branch.id)
        .fetch_optional(pool)
        .await?;

        let (has_pending_deletion, deletion_requested_at) = deletion_info.unwrap_or((false, None));

        result.push(BranchWithDeletionStatus {
            branch,
            has_pending_deletion,
            deletion_requested_at,
        });
    }

    Ok(result)
}

/// Retrieves a branch by its ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_branch_by_id(pool: &PgPool, branch_id: &str) -> Result<Option<Branch>> {
    let branch = sqlx::query_as::<_, Branch>(
        r"
        SELECT id, company_id, name, address, created_at
        FROM branches
        WHERE id = $1
        ",
    )
    .bind(branch_id)
    .fetch_optional(pool)
    .await?;

    Ok(branch)
}

/// Updates a branch's details.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn update_branch(
    pool: &PgPool,
    branch_id: &str,
    name: &str,
    address: &str,
) -> Result<Branch> {
    let updated_branch = sqlx::query_as::<_, Branch>(
        r"
        UPDATE branches
        SET name = $1, address = $2
        WHERE id = $3
        RETURNING id, company_id, name, address, created_at
        ",
    )
    .bind(name)
    .bind(address)
    .bind(branch_id)
    .fetch_one(pool)
    .await?;

    Ok(updated_branch)
}

/// Deletes a branch by its ID.
///
/// # Errors
/// Returns an error if database delete fails.
pub async fn delete_branch(pool: &PgPool, branch_id: &str) -> Result<()> {
    sqlx::query(
        r"
        DELETE FROM branches
        WHERE id = $1
        ",
    )
    .bind(branch_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a branch deletion token.
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_branch_deletion_token(
    pool: &PgPool,
    user_id: String,
    branch_id: String,
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO branch_deletion_tokens (id, user_id, branch_id, token, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&branch_id)
    .bind(&token)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Retrieves a branch deletion token record by its token.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_branch_deletion_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<(String, String, String)>> {
    let result = sqlx::query_as::<_, (String, String, String)>(
        r"
        SELECT id, user_id, branch_id
        FROM branch_deletion_tokens
        WHERE token = $1 AND used_at IS NULL AND expires_at > now()
        ",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// Marks a branch deletion token as used.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn mark_branch_deletion_token_used(pool: &PgPool, token_id: &str) -> Result<()> {
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        UPDATE branch_deletion_tokens
        SET used_at = $1
        WHERE id = $2
        ",
    )
    .bind(now)
    .bind(token_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Updates the branch association for a user.
///
/// # Errors
/// Returns an error if database update fails.
pub async fn update_user_branch(
    pool: &PgPool,
    user_id: &str,
    branch_id: Option<String>,
) -> Result<()> {
    sqlx::query(
        r"
        UPDATE users
        SET branch_id = $1
        WHERE id = $2
        ",
    )
    .bind(branch_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Accepts an invitation and creates a new user in a single transaction.
///
/// # Errors
/// Returns an error if the transaction fails, which can happen if database operations fail.
pub async fn accept_invitation_with_user_creation(
    pool: &PgPool,
    invitation_id: &str,
    email: &str,
    first_name: String,
    last_name: String,
    password_hash: String,
    company_id: &str,
    role: UserRole,
    branch_id: Option<String>,
) -> Result<UserRecord> {
    let mut tx = pool.begin().await?;

    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, branch_id, role, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "
    )
    .bind(&user_id)
    .bind(email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&password_hash)
    .bind(company_id)
    .bind(&branch_id)
    .bind(&role)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r"
        UPDATE invitations
        SET accepted_at = $1
        WHERE id = $2
        ",
    )
    .bind(now)
    .bind(invitation_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(UserRecord {
        id: user_id,
        email: email.to_string(),
        first_name,
        last_name,
        password_hash: Some(password_hash),
        company_id: Some(company_id.to_string()),
        branch_id,
        company_name: None,
        profile_picture_id: None,
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
        company_deleted_at: None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DatabaseHealthMetrics {
    pub total_connections: i64,
    pub active_connections: i64,
    pub idle_connections: i64,
    pub max_connections: i32,
    pub database_size_mb: f64,
    pub table_count: i64,
    pub index_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct SlowQueryInfo {
    pub query: String,
    pub calls: i64,
    pub total_time_ms: f64,
    pub mean_time_ms: f64,
    pub max_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct IndexUsageStats {
    pub table_name: String,
    pub index_name: String,
    pub index_scans: i64,
    pub rows_read: i64,
    pub rows_fetched: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TableSizeInfo {
    pub table_name: String,
    pub row_count: i64,
    pub total_size_mb: f64,
    pub table_size_mb: f64,
    pub index_size_mb: f64,
}

/// Retrieves various health metrics for the database.
///
/// # Errors
/// Returns an error if database queries for metrics fail.
pub async fn get_database_health(pool: &PgPool) -> Result<DatabaseHealthMetrics> {
    #[derive(sqlx::FromRow)]
    struct ConnectionStats {
        total: i64,
        active: i64,
        idle: i64,
        max_conn: i32,
    }

    #[derive(sqlx::FromRow)]
    struct DbSize {
        size_mb: f64,
    }

    #[derive(sqlx::FromRow)]
    struct TableCount {
        count: i64,
    }

    #[derive(sqlx::FromRow)]
    struct IndexCount {
        count: i64,
    }

    let conn_stats = sqlx::query_as::<_, ConnectionStats>(
        r"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE state = 'active') as active,
            COUNT(*) FILTER (WHERE state = 'idle') as idle,
            (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_conn
        FROM pg_stat_activity
        WHERE datname = current_database()
        ",
    )
    .fetch_one(pool)
    .await?;

    let db_size = sqlx::query_as::<_, DbSize>(
        r"
        SELECT (pg_database_size(current_database())::float8 / (1024.0 * 1024.0)) as size_mb
        ",
    )
    .fetch_one(pool)
    .await?;

    let table_count = sqlx::query_as::<_, TableCount>(
        r"
        SELECT COUNT(*) as count
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
        ",
    )
    .fetch_one(pool)
    .await?;

    let index_count = sqlx::query_as::<_, IndexCount>(
        r"
        SELECT COUNT(*) as count
        FROM pg_indexes
        WHERE schemaname = 'public'
        ",
    )
    .fetch_one(pool)
    .await?;

    Ok(DatabaseHealthMetrics {
        total_connections: conn_stats.total,
        active_connections: conn_stats.active,
        idle_connections: conn_stats.idle,
        max_connections: conn_stats.max_conn,
        database_size_mb: db_size.size_mb,
        table_count: table_count.count,
        index_count: index_count.count,
    })
}

/// Retrieves slow query information from `pg_stat_statements`.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_slow_queries(pool: &PgPool, limit: i64) -> Result<Vec<SlowQueryInfo>> {
    let queries = sqlx::query_as::<_, SlowQueryInfo>(
        r"
        SELECT 
            query,
            calls,
            total_exec_time as total_time_ms,
            mean_exec_time as mean_time_ms,
            max_exec_time as max_time_ms
        FROM pg_stat_statements
        WHERE query NOT LIKE '%pg_stat_statements%'
        ORDER BY mean_exec_time DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(queries)
}

/// Retrieves index usage statistics for the database.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_index_usage(pool: &PgPool) -> Result<Vec<IndexUsageStats>> {
    let stats = sqlx::query_as::<_, IndexUsageStats>(
        r"
        SELECT 
            t.relname as table_name,
            i.relname as index_name,
            x.idx_scan as index_scans,
            x.idx_tup_read as rows_read,
            x.idx_tup_fetch as rows_fetched
        FROM pg_stat_user_indexes x
        JOIN pg_class i ON i.oid = x.indexrelid
        JOIN pg_class t ON t.oid = x.relid
        ORDER BY x.idx_scan DESC
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

/// Retrieves table and index sizes for the database.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_table_sizes(pool: &PgPool) -> Result<Vec<TableSizeInfo>> {
    #[derive(sqlx::FromRow)]
    struct TableSizeRow {
        table_name: String,
        row_count: i64,
        total_size_mb: f64,
        table_size_mb: f64,
        index_size_mb: f64,
    }

    let sizes = sqlx::query_as::<_, TableSizeRow>(
        r"
        SELECT 
            relname as table_name,
            n_live_tup as row_count,
            (pg_total_relation_size(schemaname||'.'||relname)::float8 / (1024*1024)) as total_size_mb,
            (pg_relation_size(schemaname||'.'||relname)::float8 / (1024*1024)) as table_size_mb,
            (pg_indexes_size(schemaname||'.'||relname)::float8 / (1024*1024)) as index_size_mb
        FROM pg_stat_user_tables
        WHERE schemaname = 'public'
        ORDER BY pg_total_relation_size(schemaname||'.'||relname) DESC
        "
    )
    .fetch_all(pool)
    .await?;

    Ok(sizes
        .into_iter()
        .map(|s| TableSizeInfo {
            table_name: s.table_name,
            row_count: s.row_count,
            total_size_mb: s.total_size_mb,
            table_size_mb: s.table_size_mb,
            index_size_mb: s.index_size_mb,
        })
        .collect())
}

/// Identifies indexes that have never been used.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn check_unused_indexes(pool: &PgPool) -> Result<Vec<String>> {
    #[derive(sqlx::FromRow)]
    struct UnusedIndex {
        index_name: String,
    }

    let unused = sqlx::query_as::<_, UnusedIndex>(
        r"
        SELECT i.relname as index_name
        FROM pg_stat_user_indexes ui
        JOIN pg_class i ON i.oid = ui.indexrelid
        JOIN pg_class t ON t.oid = ui.relid
        WHERE ui.idx_scan = 0
        AND i.relname NOT LIKE '%_pkey%'
        ORDER BY i.relname
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(unused.into_iter().map(|u| u.index_name).collect())
}

/// Retrieves all pending invitations for a specific company.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_pending_invitations_by_company_id(
    pool: &PgPool,
    company_id: &str,
) -> Result<Vec<Invitation>> {
    let invitations = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, role, branch_id, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE company_id = $1 AND accepted_at IS NULL AND cancelled_at IS NULL AND expires_at > now()
        ",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(invitations)
}

/// Creates a new passkey session (challenge).
///
/// # Errors
/// Returns an error if database insert fails.
pub async fn create_passkey_session(
    pool: &PgPool,
    id: &str,
    session_type: &str,
    user_id: Option<String>,
    challenge: String,
    meta: Option<String>,
) -> Result<()> {
    // 5 minutes expiry
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    sqlx::query(
        r"
        INSERT INTO passkey_sessions (id, session_type, user_id, challenge, meta, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(id)
    .bind(session_type)
    .bind(user_id)
    .bind(challenge)
    .bind(meta)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves a passkey session by its ID if it hasn't expired.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_passkey_session(pool: &PgPool, id: &str) -> Result<Option<PasskeySession>> {
    let session = sqlx::query_as::<_, PasskeySession>(
        r"
        SELECT id, session_type, user_id, challenge, meta, created_at, expires_at
        FROM passkey_sessions
        WHERE id = $1 AND expires_at > NOW()
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(session)
}

/// Deletes a passkey session.
///
/// # Errors
/// Returns an error if database deletion fails.
pub async fn delete_passkey_session(pool: &PgPool, id: &str) -> Result<()> {
    sqlx::query(
        r"
        DELETE FROM passkey_sessions
        WHERE id = $1
        ",
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves all members of the same company as the requesting user.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_company_members_for_user(pool: &PgPool, user_id: &str) -> Result<Vec<UserRecord>> {
    tracing::info!("DB: Fetching members for user_id: {}", user_id);
    let users = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT target_user.id, target_user.email, target_user.first_name, target_user.last_name, 
               target_user.password_hash, target_user.company_id, target_user.branch_id, target_user.role, target_user.created_at, target_user.deleted_at, 
               companies.name as company_name, companies.deleted_at as company_deleted_at,
               target_user.oauth_provider, target_user.oauth_subject, target_user.oauth_picture, target_user.profile_picture_id
        FROM users as request_user
        JOIN users as target_user ON request_user.company_id = target_user.company_id
        LEFT JOIN companies ON target_user.company_id = companies.id
        WHERE request_user.id = $1 AND target_user.deleted_at IS NULL
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    tracing::info!("DB: Found {} members", users.len());
    for u in &users {
        tracing::info!(
            "DB: Member email={}, company_id={:?}, branch_id={:?}",
            u.email,
            u.company_id,
            u.branch_id
        );
    }

    Ok(users)
}

/// Creates a new clock-in event for a user.
///
/// # Errors
/// Returns an error if the database insert fails.
pub async fn clock_in(pool: &PgPool, user_id: &str, company_id: &str) -> Result<ClockEvent> {
    let id = Uuid::new_v4().to_string();
    let event = sqlx::query_as::<_, ClockEvent>(
        r"
        INSERT INTO clock_events (id, user_id, company_id, clock_in)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        RETURNING id, user_id, company_id, clock_in, clock_out, created_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(company_id)
    .fetch_one(pool)
    .await?;

    Ok(event)
}

/// Clocks out a user by updating the most recent open clock-in event.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn clock_out(pool: &PgPool, user_id: &str) -> Result<Option<ClockEvent>> {
    let event = sqlx::query_as::<_, ClockEvent>(
        r"
        UPDATE clock_events
        SET clock_out = CURRENT_TIMESTAMP
        WHERE id = (
            SELECT id FROM clock_events
            WHERE user_id = $1 AND clock_out IS NULL
            ORDER BY clock_in DESC
            LIMIT 1
        )
        RETURNING id, user_id, company_id, clock_in, clock_out, created_at
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(event)
}

/// Clocks out a user at a specific timestamp by updating the most recent open clock-in event.
/// This is used to cap long-running sessions (e.g., max 24 hours) at the service layer.
///
/// # Errors
/// Returns an error if the database update fails.
pub async fn clock_out_at(
    pool: &PgPool,
    user_id: &str,
    clock_out_at: chrono::DateTime<chrono::Utc>,
) -> Result<Option<ClockEvent>> {
    let event = sqlx::query_as::<_, ClockEvent>(
        r"
        UPDATE clock_events
        SET clock_out = $2
        WHERE id = (
            SELECT id FROM clock_events
            WHERE user_id = $1 AND clock_out IS NULL
            ORDER BY clock_in DESC
            LIMIT 1
        )
        RETURNING id, user_id, company_id, clock_in, clock_out, created_at
        ",
    )
    .bind(user_id)
    .bind(clock_out_at)
    .fetch_optional(pool)
    .await?;

    Ok(event)
}

/// Gets the current clock status for a user (latest event).
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_clock_status(pool: &PgPool, user_id: &str) -> Result<Option<ClockEvent>> {
    let event = sqlx::query_as::<_, ClockEvent>(
        r"
        SELECT id, user_id, company_id, clock_in, clock_out, created_at
        FROM clock_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(event)
}

/// Gets the last N clock events for a user.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_recent_clock_events(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<ClockEvent>> {
    let events = sqlx::query_as::<_, ClockEvent>(
        r"
        SELECT id, user_id, company_id, clock_in, clock_out, created_at
        FROM clock_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        ",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(events)
}

/// A clock event row joined with user info, for company-wide reporting.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CompanyClockEventRow {
    pub id: String,
    pub user_id: String,
    pub company_id: String,
    pub clock_in: chrono::DateTime<chrono::Utc>,
    pub clock_out: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl CompanyClockEventRow {
    #[must_use]
    pub fn is_clocked_in(&self) -> bool {
        self.clock_out.is_none()
    }
}

/// Gets all clock events for a company, joined with user name/email.
/// Optionally filtered by date range and branch.
///
/// # Errors
/// Returns an error if the database query fails.
pub async fn get_company_clock_events(
    pool: &PgPool,
    company_id: &str,
    from: Option<chrono::DateTime<chrono::Utc>>,
    to: Option<chrono::DateTime<chrono::Utc>>,
    branch_id: Option<String>,
) -> Result<Vec<CompanyClockEventRow>> {
    // Filter out empty strings from branch_id
    let branch_id = branch_id.filter(|s| !s.is_empty());

    let mut query_str = String::from(
        r"
        SELECT ce.id, ce.user_id, ce.company_id, ce.clock_in, ce.clock_out,
               ce.created_at,
               u.first_name, u.last_name, u.email
        FROM clock_events ce
        JOIN users u ON u.id = ce.user_id
        WHERE ce.company_id = $1
        ",
    );

    let mut bind_count = 1;

    // Add branch filter if provided
    if branch_id.is_some() {
        bind_count += 1;
        writeln!(query_str, "  AND u.branch_id = ${bind_count}")?;
    }

    // Add date filters
    if from.is_some() {
        bind_count += 1;
        writeln!(query_str, "  AND ce.clock_in >= ${bind_count}")?;
    }

    if to.is_some() {
        bind_count += 1;
        writeln!(query_str, "  AND ce.clock_in <= ${bind_count}")?;
    }

    query_str.push_str("ORDER BY ce.clock_in DESC");

    // Log the query for debugging
    tracing::debug!("Clock events query: {}", query_str);
    tracing::debug!("Branch ID filter: {:?}", branch_id);

    let mut query = sqlx::query_as::<_, CompanyClockEventRow>(&query_str).bind(company_id);

    if let Some(bid) = branch_id {
        query = query.bind(bid);
    }

    if let Some(f) = from {
        query = query.bind(f);
    }

    if let Some(t) = to {
        query = query.bind(t);
    }

    let events = query.fetch_all(pool).await?;

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_clocked_in_with_future_clock_in() {
        let future_time = chrono::Utc::now() + chrono::Duration::seconds(60);
        let event = ClockEvent {
            id: "test".to_string(),
            user_id: "user1".to_string(),
            company_id: "company1".to_string(),
            clock_in: future_time,
            clock_out: None,
            created_at: chrono::Utc::now(),
        };
        assert!(
            event.is_clocked_in(),
            "ClockEvent with clock_out=None should be clocked in even if clock_in is in the future"
        );
    }

    #[test]
    fn test_is_clocked_in_with_past_clock_in() {
        let past_time = chrono::Utc::now() - chrono::Duration::seconds(60);
        let event = ClockEvent {
            id: "test".to_string(),
            user_id: "user1".to_string(),
            company_id: "company1".to_string(),
            clock_in: past_time,
            clock_out: None,
            created_at: chrono::Utc::now(),
        };
        assert!(
            event.is_clocked_in(),
            "ClockEvent with clock_out=None should be clocked in"
        );
    }

    #[test]
    fn test_is_clocked_in_with_clock_out() {
        let past_time = chrono::Utc::now() - chrono::Duration::seconds(60);
        let event = ClockEvent {
            id: "test".to_string(),
            user_id: "user1".to_string(),
            company_id: "company1".to_string(),
            clock_in: past_time,
            clock_out: Some(past_time + chrono::Duration::seconds(60)),
            created_at: chrono::Utc::now(),
        };
        assert!(
            !event.is_clocked_in(),
            "ClockEvent with clock_out set should not be clocked in"
        );
    }

    #[test]
    fn test_company_clock_event_row_is_clocked_in() {
        let future_time = chrono::Utc::now() + chrono::Duration::seconds(60);
        let row = CompanyClockEventRow {
            id: "test".to_string(),
            user_id: "user1".to_string(),
            company_id: "company1".to_string(),
            clock_in: future_time,
            clock_out: None,
            created_at: chrono::Utc::now(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(
            row.is_clocked_in(),
            "CompanyClockEventRow with clock_out=None should be clocked in even if clock_in is in the future"
        );
    }
}
