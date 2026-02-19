use crate::{
    db::{self, UserRole},
    logs_db,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminUpdateMemberRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "Jane")]
    pub first_name: String,
    #[schema(example = "Smith")]
    pub last_name: String,
    #[schema(example = "staff")]
    pub role: String,
    #[schema(example = "branch-uuid-here")]
    pub branch_id: Option<String>,
}
#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelInvitationRequest {
    #[schema(example = "invitation-uuid-here")]
    pub invitation_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveMemberRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<String>,
    pub branch_id: Option<String>,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::db::UserRecord> for UserDto {
    fn from(user: crate::db::UserRecord) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            company_id: user.company_id,
            branch_id: user.branch_id,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BranchDto {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub has_pending_deletion: bool,
    #[serde(default)]
    pub deletion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<db::Branch> for BranchDto {
    fn from(branch: db::Branch) -> Self {
        Self {
            id: branch.id,
            company_id: branch.company_id,
            name: branch.name,
            address: branch.address,
            created_at: branch.created_at,
            has_pending_deletion: false,
            deletion_requested_at: None,
        }
    }
}

impl From<db::BranchWithDeletionStatus> for BranchDto {
    fn from(branch_with_status: db::BranchWithDeletionStatus) -> Self {
        Self {
            id: branch_with_status.branch.id,
            company_id: branch_with_status.branch.company_id,
            name: branch_with_status.branch.name,
            address: branch_with_status.branch.address,
            created_at: branch_with_status.branch.created_at,
            has_pending_deletion: branch_with_status.has_pending_deletion,
            deletion_requested_at: branch_with_status.deletion_requested_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBranchRequest {
    #[schema(example = "London Office")]
    pub name: String,
    #[schema(example = "123 Regent St, London")]
    pub address: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBranchRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub branch_id: String,
    #[schema(example = "London Office")]
    pub name: String,
    #[schema(example = "123 Regent St, London")]
    pub address: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestBranchDeletionRequest {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub branch_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RequestBranchDeletionResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfirmBranchDeletionRequest {
    #[schema(example = "deletion-token-here")]
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfirmBranchDeletionResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListBranchesResponse {
    pub branches: Vec<BranchDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyDto {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvitationDto {
    pub id: String,
    pub company_id: String,
    pub email: String,
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetPendingInvitationsResponse {
    pub invitations: Vec<InvitationResponse>,
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
    pub created_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "SecurePass123!")]
    pub password: String,
    #[schema(example = "Example Corp")]
    pub company_name: String,
    #[schema(example = "123 Main St, City, Country")]
    pub company_address: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetCompanyMembersResponse {
    pub members: Vec<UserResponse>,
}

impl From<Vec<db::UserRecord>> for GetCompanyMembersResponse {
    fn from(members: Vec<db::UserRecord>) -> Self {
        Self {
            members: members.into_iter().map(UserResponse::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct VerifyTokenRequest {
    #[schema(example = "jwt-token-here")]
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddTemplateRequest {
    #[schema(example = "Kitchen Daily Log")]
    pub template_name: String,
    #[schema(example = "[\"field1\", \"field2\"]")]
    pub template_layout: logs_db::TemplateLayout,
    #[schema(example = "{\"frequency\": \"daily\", \"time\": \"08:00\"}")]
    pub schedule: logs_db::Schedule,
    #[schema(example = "branch-uuid-here")]
    pub branch_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateTemplateResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTemplateRequest {
    #[schema(example = "Kitchen Daily Log")]
    pub template_name: String,
    #[schema(example = "[\"field1\", \"field2\"]")]
    pub template_layout: Option<logs_db::TemplateLayout>,
    #[schema(example = "{\"frequency\": \"daily\", \"time\": \"08:00\"}")]
    pub schedule: Option<logs_db::Schedule>,
    #[schema(example = "Major Update")]
    pub version_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TemplateVersionInfo {
    pub version: u16,
    pub version_name: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetTemplateVersionsResponse {
    pub versions: Vec<TemplateVersionInfo>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RestoreTemplateVersionRequest {
    pub version: u16,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetTemplateRequest {
    #[schema(example = "Kitchen Daily Log")]
    pub template_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TemplateInfo {
    pub template_name: String,
    pub schedule: logs_db::Schedule,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetAllTemplatesResponse {
    pub templates: Vec<TemplateInfo>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InviteUserRequest {
    #[schema(example = "newmember@example.com")]
    pub email: String,
    #[schema(example = "staff")]
    pub role: Option<UserRole>,
    #[schema(example = "branch-uuid-here")]
    pub branch_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitationRequest {
    #[schema(example = "invitation-token-here")]
    pub token: String,
    #[schema(example = "Alice")]
    pub first_name: String,
    #[schema(example = "Smith")]
    pub last_name: String,
    #[schema(example = "MemberPass123!")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JwtVerifyResponse {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: Option<String>,
    pub company_name: Option<String>,
    pub branch_id: Option<String>,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub oauth_provider: Option<String>,
}

impl From<db::UserRecord> for UserResponse {
    fn from(user: db::UserRecord) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            company_id: user.company_id,
            company_name: user.company_name,
            branch_id: user.branch_id,
            role: user.role,
            created_at: user.created_at,
            oauth_provider: user.oauth_provider,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetInvitationDetailsRequest {
    #[schema(example = "invitation-token-here")]
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetInvitationDetailsResponse {
    pub company_name: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImageUploadResponse {
    pub filename: String,
    pub object_id: String,
    pub file_size: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestPasswordResetRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AddTokenResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AddTemplateResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetTemplateResponse {
    pub template_name: String,
    pub template_layout: logs_db::TemplateLayout,
    pub version: u16,
    pub version_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasswordResetResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    #[schema(example = "reset-token-here")]
    pub token: String,
    #[schema(example = "NewPassword123!")]
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthInitiateResponse {
    pub authorization_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthCallbackRequest {
    #[schema(example = "authorization-code-from-google")]
    pub code: String,
    #[schema(example = "csrf-state-token")]
    pub state: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLogEntryRequest {
    #[schema(example = "Kitchen Daily Log")]
    pub template_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogEntryResponse {
    pub id: String,
    pub template_name: String,
    pub template_layout: logs_db::TemplateLayout,
    pub entry_data: serde_json::Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub submitted_at: Option<String>,
    pub period: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLogEntryRequest {
    pub entry_data: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitLogEntryRequest {}

#[derive(Debug, Serialize, ToSchema)]
pub struct DueFormInfo {
    pub template_name: String,
    pub template_layout: logs_db::TemplateLayout,
    pub last_submitted: Option<String>,
    pub period: String,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DueFormsResponse {
    pub forms: Vec<DueFormInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListLogEntriesResponse {
    pub entries: Vec<LogEntryResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateLogEntryResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmitLogEntryResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RenameTemplateRequest {
    pub old_template_name: String,
    pub new_template_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct RenameTemplateResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct DeleteTemplateRequest {
    pub template_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteTemplateResponse {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LayoutGenerationRequest {
    pub user_prompt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LayoutGenerationResponse {
    pub layout: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyRegistrationStartRequest {
    #[schema(example = "My MacBook Pro")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeyRegistrationStartResponse {
    pub options: serde_json::Value,
    pub auth_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyRegistrationFinishRequest {
    pub credential: serde_json::Value,
    pub auth_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyAuthenticationStartRequest {
    #[schema(example = "user@example.com")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeyAuthenticationStartResponse {
    pub options: serde_json::Value,
    pub auth_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyAuthenticationFinishRequest {
    pub credential: serde_json::Value,
    pub auth_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListPasskeysResponse {
    pub passkeys: Vec<PasskeyDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeyDto {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<db::Passkey> for PasskeyDto {
    fn from(passkey: db::Passkey) -> Self {
        Self {
            id: passkey.id,
            name: passkey.name,
            created_at: passkey.created_at,
            last_used_at: passkey.last_used_at,
        }
    }
}
