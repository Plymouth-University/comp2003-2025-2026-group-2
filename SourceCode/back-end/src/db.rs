use sqlx::SqlitePool;
use anyhow::Result;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "member")]
    Member,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "member" => Ok(UserRole::Member),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: String,
    pub company_id: Option<String>,
    pub role: String,
    pub created_at: String,
}

impl User {
    pub fn get_role(&self) -> UserRole {
        self.role.parse().unwrap_or(UserRole::Member)
    }

    pub fn is_admin(&self) -> bool {
        self.get_role() == UserRole::Admin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invitation {
    pub id: String,
    pub company_id: String,
    pub email: String,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
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
    pub created_at: String,
}

pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS companies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            company_id TEXT,
            role TEXT NOT NULL DEFAULT 'member',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (company_id) REFERENCES companies(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invitations (
            id TEXT PRIMARY KEY,
            company_id TEXT NOT NULL,
            email TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            accepted_at DATETIME,
            FOREIGN KEY (company_id) REFERENCES companies(id),
            UNIQUE(company_id, email)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS security_logs (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            user_id TEXT,
            email TEXT,
            ip_address TEXT,
            user_agent TEXT,
            details TEXT,
            success INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_security_logs_event_type ON security_logs(event_type)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_security_logs_user_id ON security_logs(user_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_security_logs_created_at ON security_logs(created_at)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_company_id ON users(company_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_invitations_company_id ON invitations(company_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_invitations_email ON invitations(email)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_invitations_token ON invitations(token)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_user(
    pool: &SqlitePool,
    email: String,
    first_name: String,
    last_name: String,
    password_hash: String,
    company_id: Option<String>,
    role: UserRole,
) -> Result<User> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let role_str = role.to_string();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        id,
        email,
        first_name,
        last_name,
        password_hash,
        company_id,
        role_str,
        now
    )
    .execute(pool)
    .await?;

    Ok(User {
        id,
        email,
        first_name,
        last_name,
        password_hash,
        company_id,
        role: role_str,
        created_at: now,
    })
}

pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id as "id!", email as "email!", first_name as "first_name!", last_name as "last_name!", 
               password_hash as "password_hash!", company_id, role as "role!", created_at as "created_at!: String"
        FROM users
        WHERE email = ?
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id as "id!", email as "email!", first_name as "first_name!", last_name as "last_name!", 
               password_hash as "password_hash!", company_id, role as "role!", created_at as "created_at!: String"
        FROM users
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
pub async fn create_company(
    pool: &SqlitePool,
    name: String,
    address: String,
) -> Result<Company> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO companies (id, name, address, created_at)
        VALUES (?, ?, ?, ?)
        "#,
        id,
        name,
        address,
        now
    )
    .execute(pool)
    .await?;

    Ok(Company {
        id,
        name,
        address,
        created_at: now,
    })
}

pub async fn get_company_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Company>> {
    let company = sqlx::query_as!(
        Company,
        r#"
        SELECT id as "id!", name as "name!", address as "address!", created_at as "created_at!: String"
        FROM companies
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(company)
}

pub async fn create_invitation(
    pool: &SqlitePool,
    company_id: String,
    email: String,
    token: String,
    expires_at: String,
) -> Result<Invitation> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO invitations (id, company_id, email, token, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        id,
        company_id,
        email,
        token,
        now,
        expires_at
    )
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
    })
}

pub async fn get_invitation_by_token(pool: &SqlitePool, token: &str) -> Result<Option<Invitation>> {
    let invitation = sqlx::query_as!(
        Invitation,
        r#"
        SELECT id as "id!", company_id as "company_id!", email as "email!", token as "token!", 
               created_at as "created_at!: String", expires_at as "expires_at!: String", accepted_at as "accepted_at: String"
        FROM invitations
        WHERE token = ? AND accepted_at IS NULL
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

pub async fn accept_invitation(
    pool: &SqlitePool,
    invitation_id: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        UPDATE invitations
        SET accepted_at = ?
        WHERE id = ?
        "#,
        now,
        invitation_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn log_security_event(
    pool: &SqlitePool,
    event_type: String,
    user_id: Option<String>,
    email: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    details: Option<String>,
    success: bool,
) -> Result<SecurityLog> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let success_int = if success { 1 } else { 0 };

    sqlx::query!(
        r#"
        INSERT INTO security_logs (id, event_type, user_id, email, ip_address, user_agent, details, success, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        id,
        event_type,
        user_id,
        email,
        ip_address,
        user_agent,
        details,
        success_int,
        now
    )
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
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<SecurityLog>> {
    let logs = sqlx::query_as::<_, SecurityLog>(
        r#"
        SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
        FROM security_logs
        WHERE user_id = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

pub async fn get_recent_security_logs(
    pool: &SqlitePool,
    event_type: Option<String>,
    limit: i64,
) -> Result<Vec<SecurityLog>> {
    let logs = if let Some(evt) = event_type {
        sqlx::query_as::<_, SecurityLog>(
            r#"
            SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
            FROM security_logs
            WHERE event_type = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(evt)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, SecurityLog>(
            r#"
            SELECT id, event_type, user_id, email, ip_address, user_agent, details, success, created_at
            FROM security_logs
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(logs)
}
