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

    sqlx::query(
        r#"
        INSERT INTO users (id, email, first_name, last_name, password_hash, company_id, role, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&email)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&password_hash)
    .bind(&company_id)
    .bind(&role_str)
    .bind(&now)
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
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, first_name, last_name, password_hash, company_id, role, created_at
        FROM users
        WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, first_name, last_name, password_hash, company_id, role, created_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(id)
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

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, address, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&address)
    .bind(&now)
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
    let company = sqlx::query_as::<_, Company>(
        r#"
        SELECT id, name, address, created_at
        FROM companies
        WHERE id = ?
        "#,
    )
    .bind(id)
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

    sqlx::query(
        r#"
        INSERT INTO invitations (id, company_id, email, token, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&company_id)
    .bind(&email)
    .bind(&token)
    .bind(&now)
    .bind(&expires_at)
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
    let invitation = sqlx::query_as::<_, Invitation>(
        r#"
        SELECT id, company_id, email, token, created_at, expires_at, accepted_at
        FROM invitations
        WHERE token = ? AND accepted_at IS NULL
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(invitation)
}

pub async fn accept_invitation(
    pool: &SqlitePool,
    invitation_id: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE invitations
        SET accepted_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&now)
    .bind(invitation_id)
    .execute(pool)
    .await?;

    Ok(())
}
