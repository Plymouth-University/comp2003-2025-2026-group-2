use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "member")]
    Member,
    #[serde(rename = "logsmart_admin")]
    #[sqlx(rename = "logsmart_admin")]
    LogSmartAdmin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Member => write!(f, "member"),
            UserRole::LogSmartAdmin => write!(f, "logsmart_admin"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "member" => Ok(UserRole::Member),
            "logsmart_admin" => Ok(UserRole::LogSmartAdmin),
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
    pub fn is_admin(&self) -> bool {
        self.get_role() == UserRole::Admin
    }

    #[must_use]
    pub fn is_member(&self) -> bool {
        self.get_role() == UserRole::Member
    }

    #[must_use]
    pub fn is_logsmart_admin(&self) -> bool {
        self.get_role() == UserRole::LogSmartAdmin
    }

    #[must_use]
    pub fn is_company_admin(&self) -> bool {
        self.is_admin()
    }

    #[must_use]
    pub fn can_manage_company(&self) -> bool {
        self.is_admin() || self.is_logsmart_admin()
    }
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

pub async fn init_db(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r"
        DO $$ BEGIN
            CREATE TYPE user_role AS ENUM ('admin', 'member', 'logsmart_admin');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS companies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            password_hash TEXT,
            company_id TEXT,
            role user_role NOT NULL DEFAULT 'member',
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            deleted_at TIMESTAMPTZ,
            oauth_provider TEXT,
            oauth_subject TEXT,
            oauth_picture TEXT,
            FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE SET NULL,
            CONSTRAINT valid_email CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
            CONSTRAINT password_or_oauth CHECK (password_hash IS NOT NULL OR oauth_provider IS NOT NULL),
            CONSTRAINT unique_oauth UNIQUE (oauth_provider, oauth_subject)
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_provider TEXT
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_subject TEXT
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_picture TEXT
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS invitations (
            id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            email TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            expires_at TIMESTAMPTZ NOT NULL,
            accepted_at TIMESTAMPTZ,
            cancelled_at TIMESTAMPTZ,
            FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
            CONSTRAINT valid_invitation_email CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
            CONSTRAINT valid_expiry CHECK (expires_at > created_at)
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_active_invitations_company_email
        ON invitations (company_id, email)
        WHERE cancelled_at IS NULL AND accepted_at IS NULL
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS security_logs (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            user_id TEXT,
            email TEXT,
            ip_address TEXT,
            user_agent TEXT,
            details TEXT,
            success BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_security_logs_event_type ON security_logs(event_type)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_security_logs_user_id ON security_logs(user_id)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_security_logs_created_at ON security_logs(created_at)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_security_logs_failed_logins 
        ON security_logs(email, created_at) 
        WHERE event_type = 'login' AND success = false
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_users_company_id ON users(company_id)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_invitations_company_id ON invitations(company_id)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_invitations_email ON invitations(email)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_invitations_token ON invitations(token)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_invitations_active 
        ON invitations(company_id, email, expires_at) 
        WHERE accepted_at IS NULL
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS password_resets (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            expires_at TIMESTAMPTZ NOT NULL,
            used_at TIMESTAMPTZ,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            CONSTRAINT valid_reset_expiry CHECK (expires_at > created_at)
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_password_resets_user_id ON password_resets(user_id)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_password_resets_token ON password_resets(token)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_password_resets_active 
        ON password_resets(token, expires_at) 
        WHERE used_at IS NULL
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS passkeys (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            credential_id TEXT NOT NULL UNIQUE,
            public_key TEXT NOT NULL,
            counter BIGINT NOT NULL DEFAULT 0,
            name TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            last_used_at TIMESTAMPTZ,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id)
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS passkey_sessions (
            id TEXT PRIMARY KEY,
            session_type TEXT NOT NULL,
            user_id TEXT,
            challenge TEXT NOT NULL,
            meta TEXT,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            expires_at TIMESTAMPTZ NOT NULL
        )
        ",
    )
    .execute(pool)
    .await?;

    Ok(())
}

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
        company_name: None,
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: None,
        oauth_subject: None,
        oauth_picture: None,
    })
}

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

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.role, users.created_at, users.deleted_at, 
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

pub async fn get_user_by_id(pool: &PgPool, id: &str) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
            users.password_hash, users.company_id, users.role, users.created_at, users.deleted_at, 
            companies.name as company_name, users.oauth_provider, users.oauth_subject, users.oauth_picture
        FROM users
        LEFT JOIN companies ON users.company_id = companies.id
        WHERE users.id = $1 AND users.deleted_at IS NULL
        "
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_oauth(
    pool: &PgPool,
    provider: &str,
    subject: &str,
) -> Result<Option<UserRecord>> {
    let user = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
            users.password_hash, users.company_id, users.role, users.created_at, users.deleted_at, 
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
        company_name: None,
        role,
        created_at: now,
        deleted_at: None,
        oauth_provider: Some(oauth_provider),
        oauth_subject: Some(oauth_subject),
        oauth_picture,
    })
}

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

pub async fn create_invitation(
    pool: &PgPool,
    company_id: String,
    email: String,
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<Invitation> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO invitations (id, company_id, email, token, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(&id)
    .bind(&company_id)
    .bind(&email)
    .bind(&token)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(Invitation {
        id,
        company_id,
        email,
        token,
        created_at: now,
        expires_at,
        accepted_at: None,
        cancelled_at: None,
    })
}

pub async fn get_invitation_by_token(pool: &PgPool, token: &str) -> Result<Option<Invitation>> {
    let invitation = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE token = $1 AND accepted_at IS NULL AND cancelled_at IS NULL
        ",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

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

pub async fn get_invitation_by_id(
    pool: &PgPool,
    invitation_id: &str,
) -> Result<Option<Invitation>> {
    let invitation = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE id = $1
        ",
    )
    .bind(invitation_id)
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

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

pub async fn update_user_profile_full(
    pool: &PgPool,
    user_id: &str,
    first_name: String,
    last_name: String,
    role: UserRole,
) -> Result<UserRecord> {
    sqlx::query(
        r"
        UPDATE users
        SET first_name = $1, last_name = $2, role = $3
        WHERE id = $4
        ",
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(&role)
    .bind(user_id)
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(user)
}

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

pub async fn get_users_by_company_id(pool: &PgPool, company_id: &str) -> Result<Vec<UserRecord>> {
    let users = sqlx::query_as::<_, UserRecord>(
        r"
        SELECT users.id, users.email, users.first_name, users.last_name, 
               users.password_hash, users.company_id, users.role, users.created_at, users.deleted_at, companies.name as company_name
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

pub async fn accept_invitation_with_user_creation(
    pool: &PgPool,
    invitation_id: &str,
    email: &str,
    first_name: String,
    last_name: String,
    password_hash: String,
    company_id: &str,
) -> Result<UserRecord> {
    let mut tx = pool.begin().await?;

    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        r"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "
    )
    .bind(&user_id)
    .bind(email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&password_hash)
    .bind(company_id)
    .bind(UserRole::Member)
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
        company_name: None,
        role: UserRole::Member,
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
        SELECT pg_database_size(current_database()) / (1024.0 * 1024.0) as size_mb
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

pub async fn get_index_usage(pool: &PgPool) -> Result<Vec<IndexUsageStats>> {
    let stats = sqlx::query_as::<_, IndexUsageStats>(
        r"
        SELECT 
            schemaname || '.' || tablename as table_name,
            indexname as index_name,
            idx_scan as index_scans,
            idx_tup_read as rows_read,
            idx_tup_fetch as rows_fetched
        FROM pg_stat_user_indexes
        WHERE schemaname = 'public'
        ORDER BY idx_scan DESC
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

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
            tablename as table_name,
            n_live_tup as row_count,
            pg_total_relation_size(schemaname||'.'||tablename)::numeric / (1024*1024) as total_size_mb,
            pg_relation_size(schemaname||'.'||tablename)::numeric / (1024*1024) as table_size_mb,
            pg_indexes_size(schemaname||'.'||tablename)::numeric / (1024*1024) as index_size_mb
        FROM pg_stat_user_tables
        WHERE schemaname = 'public'
        ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
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

pub async fn check_unused_indexes(pool: &PgPool) -> Result<Vec<String>> {
    #[derive(sqlx::FromRow)]
    struct UnusedIndex {
        index_name: String,
    }

    let unused = sqlx::query_as::<_, UnusedIndex>(
        r"
        SELECT indexname as index_name
        FROM pg_stat_user_indexes
        WHERE schemaname = 'public'
        AND idx_scan = 0
        AND indexname NOT LIKE '%_pkey'
        ORDER BY indexname
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(unused.into_iter().map(|u| u.index_name).collect())
}

pub async fn get_pending_invitations_by_company_id(
    pool: &PgPool,
    company_id: &str,
) -> Result<Vec<Invitation>> {
    let invitations = sqlx::query_as::<_, Invitation>(
        r"
        SELECT id, company_id, email, token, created_at, expires_at, accepted_at, cancelled_at
        FROM invitations
        WHERE company_id = $1 AND accepted_at IS NULL AND cancelled_at IS NULL AND expires_at > now()
        ",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    Ok(invitations)
}

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
