use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub oauth_picture: Option<String>,
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
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
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
               companies.name as company_name, users.oauth_provider, users.oauth_subject, users.oauth_picture
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.email = $1 AND users.deleted_at IS NULL
        "
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
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
            companies.name as company_name, users.oauth_provider, users.oauth_subject, users.oauth_picture
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.id = $1 AND users.deleted_at IS NULL
        "
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if let Some(u) = &user {
        tracing::info!(
            "DB: Found user {} with company_id: {:?}, branch_id: {:?}",
            u.email,
            u.company_id,
            u.branch_id
        );
    } else {
        tracing::info!("DB: User {} not found", id);
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
            companies.name as company_name, users.oauth_provider, users.oauth_subject, users.oauth_picture
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.oauth_provider = $1 AND users.oauth_subject = $2 AND users.deleted_at IS NULL
        "
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
        oauth_provider: Some(oauth_provider),
        oauth_subject: Some(oauth_subject),
        oauth_picture,
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
    })
}

/// Retrieves a company by its ID.
///
/// # Errors
/// Returns an error if database query fails.
pub async fn get_company_by_id(pool: &PgPool, id: &str) -> Result<Option<Company>> {
    let company = sqlx::query_as::<_, Company>(
        r"
        SELECT id, name, address, created_at
        FROM companies
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

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
        RETURNING id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
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
) -> Result<UserRecord> {
    sqlx::query(
        r"
        UPDATE users
        SET first_name = $1, last_name = $2, role = $3, branch_id = $4
        WHERE id = $5
        ",
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(&role)
    .bind(branch_id)
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
               companies.name as company_name, users.oauth_provider, users.oauth_subject, users.oauth_picture
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
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
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
               companies.name as company_name, target_user.oauth_provider, target_user.oauth_subject, target_user.oauth_picture
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
        INSERT INTO clock_events (id, user_id, company_id, clock_in, status)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, 'in')
        RETURNING id, user_id, company_id, clock_in, clock_out, status, created_at
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
        SET clock_out = CURRENT_TIMESTAMP, status = 'out'
        WHERE id = (
            SELECT id FROM clock_events
            WHERE user_id = $1 AND status = 'in'
            ORDER BY clock_in DESC
            LIMIT 1
        )
        RETURNING id, user_id, company_id, clock_in, clock_out, status, created_at
        ",
    )
    .bind(user_id)
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
        SELECT id, user_id, company_id, clock_in, clock_out, status, created_at
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
        SELECT id, user_id, company_id, clock_in, clock_out, status, created_at
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
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
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
               ce.status, ce.created_at,
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
        query_str.push_str(&format!("  AND u.branch_id = ${}\n", bind_count));
    }

    // Add date filters
    if from.is_some() {
        bind_count += 1;
        query_str.push_str(&format!("  AND ce.clock_in >= ${}\n", bind_count));
    }

    if to.is_some() {
        bind_count += 1;
        query_str.push_str(&format!("  AND ce.clock_in <= ${}\n", bind_count));
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
mod db_model_tests {
    use super::*;
    use chrono::{Duration, Utc};

    // UserRole Enum Tests
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
        // Test that parsing is case sensitive
        assert!("ADMIN".parse::<UserRole>().is_err());
        assert!("Admin".parse::<UserRole>().is_err());
        assert!("COMPANY_MANAGER".parse::<UserRole>().is_err());

        // Only lowercase should work
        assert!("company_manager".parse::<UserRole>().is_ok());
    }

    #[test]
    fn test_user_role_serde_compatibility() {
        // Test that serde attributes work correctly
        let role = UserRole::BranchManager;
        let serialized = serde_json::to_string(&role).unwrap();
        assert!(serialized.contains("\"branch_manager\""));

        let deserialized: UserRole = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, UserRole::BranchManager);
    }

    // UserRecord Tests
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
        assert!(!user.can_manage_company()); // Members need company to manage
    }

    // Company Tests
    #[test]
    fn test_company_creation() {
        let now = Utc::now();
        let company = Company {
            id: "company1".to_string(),
            name: "Test Company".to_string(),
            address: "123 Test St".to_string(),
            created_at: now,
        };

        assert_eq!(company.id, "company1");
        assert_eq!(company.name, "Test Company");
        assert_eq!(company.address, "123 Test St");
        assert_eq!(company.created_at, now);
    }

    // Invitation Tests
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
            expires_at: now - Duration::hours(1), // Expired 1 hour ago
            ..create_test_invitation()
        };

        let valid_invitation = Invitation {
            expires_at: now + Duration::hours(1), // Expires in 1 hour
            ..create_test_invitation()
        };

        assert!(expired_invitation.expires_at < now);
        assert!(valid_invitation.expires_at > now);
    }

    // SecurityLog Tests
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
        };

        assert!(log.user_id.is_none());
        assert!(log.email.is_none());
        assert!(log.ip_address.is_none());
        assert!(log.user_agent.is_none());
        assert!(log.details.is_none());
        assert!(log.success);
    }

    // Passkey Tests
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

    // PasskeySession Tests
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
            expires_at: now - Duration::minutes(1), // Expired 1 minute ago
            ..create_test_passkey_session()
        };

        let valid_session = PasskeySession {
            expires_at: now + Duration::minutes(5), // Expires in 5 minutes
            ..create_test_passkey_session()
        };

        assert!(expired_session.expires_at < now);
        assert!(valid_session.expires_at > now);
    }

    // UserDisplay Tests
    #[test]
    fn test_user_display_creation() {
        let display = UserDisplay {
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

    // Helper Functions for Testing
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

    // Complex Scenario Tests
    #[test]
    fn test_user_lifecycle_scenarios() {
        let now = Utc::now();

        // New user creation
        let new_user = UserRecord {
            created_at: now,
            deleted_at: None,
            password_hash: Some("hash".to_string()),
            ..create_test_user_record()
        };

        assert!(new_user.password_hash.is_some());
        assert!(new_user.deleted_at.is_none());

        // User deletion
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

        // Initial invitation
        let base_invitation = create_test_invitation();
        let invitation = Invitation {
            created_at: now,
            accepted_at: None,
            cancelled_at: None,
            ..base_invitation.clone()
        };

        assert!(invitation.accepted_at.is_none());
        assert!(invitation.cancelled_at.is_none());

        // Accepted invitation
        let accepted_invitation = Invitation {
            accepted_at: Some(now + Duration::hours(1)),
            ..base_invitation.clone()
        };

        assert!(accepted_invitation.accepted_at.is_some());

        // Cancelled invitation
        let cancelled_invitation = Invitation {
            cancelled_at: Some(now + Duration::hours(2)),
            ..base_invitation
        };

        assert!(cancelled_invitation.cancelled_at.is_some());
    }

    // Database Constraint Validation Tests
    #[test]
    fn test_user_email_format_validation() {
        // Test various email formats that should be valid according to database constraint
        let valid_emails = vec![
            "user@example.com",
            "test.email+tag@domain.co.uk",
            "user_name123@test-domain.com",
            "a@b.co", // Minimal valid email
        ];

        for email in valid_emails {
            // This tests the regex pattern used in database constraint
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
        // Test invitation email constraint (same as user email)
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
}
