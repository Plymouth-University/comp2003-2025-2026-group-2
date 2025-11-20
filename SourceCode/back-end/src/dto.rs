use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<String>,
    pub role: String,
    pub created_at: String,
}

impl From<crate::db::UserRecord> for UserDto {
    fn from(user: crate::db::UserRecord) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            company_id: user.company_id,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyDto {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: String,
}

impl From<crate::db::Company> for CompanyDto {
    fn from(company: crate::db::Company) -> Self {
        Self {
            id: company.id,
            name: company.name,
            address: company.address,
            created_at: company.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationDto {
    pub id: String,
    pub company_id: String,
    pub email: String,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
}

impl From<crate::db::Invitation> for InvitationDto {
    fn from(invitation: crate::db::Invitation) -> Self {
        Self {
            id: invitation.id,
            company_id: invitation.company_id,
            email: invitation.email,
            token: invitation.token,
            created_at: invitation.created_at,
            expires_at: invitation.expires_at,
            accepted_at: invitation.accepted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLogDto {
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

impl From<crate::db::SecurityLog> for SecurityLogDto {
    fn from(log: crate::db::SecurityLog) -> Self {
        Self {
            id: log.id,
            event_type: log.event_type,
            user_id: log.user_id,
            email: log.email,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            details: log.details,
            success: log.success,
            created_at: log.created_at,
        }
    }
}
