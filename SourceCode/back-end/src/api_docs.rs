use crate::handlers;
use crate::logs_db;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::register_company_admin,
        handlers::login,
        handlers::verify_token,
        handlers::get_current_user,
        handlers::invite_user,
        handlers::accept_invitation,
        handlers::update_profile,
        handlers::request_password_reset,
        handlers::reset_password,
        handlers::add_template,
        handlers::get_template,
        handlers::get_company_members,
    ),
    components(
        schemas(
            handlers::RegisterRequest,
            handlers::LoginRequest,
            handlers::InviteUserRequest,
            handlers::AcceptInvitationRequest,
            handlers::VerifyTokenRequest,
            handlers::AuthResponse,
            handlers::UserResponse,
            handlers::InvitationResponse,
            handlers::ErrorResponse,
            handlers::JwtVerifyResponse,
            handlers::UpdateProfileRequest,
            handlers::RequestPasswordResetRequest,
            handlers::ResetPasswordRequest,
            handlers::PasswordResetResponse,
            logs_db::TemplateDocument,
            logs_db::TemplateField,
            logs_db::Position,
            handlers::AddTemplateRequest,
            handlers::GetTemplateRequest,
            handlers::GetTemplateResponse,
            handlers::AddTemplateResponse,
            handlers::AddTokenResponse,
            handlers::GetCompanyMembersResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and registration endpoints"),
        (name = "Invitations", description = "Company member invitation management"),
        (name = "Templates", description = "Template1 management"),
        (name = "Company Management", description = "Company member management"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
