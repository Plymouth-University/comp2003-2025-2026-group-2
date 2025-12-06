use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};

use crate::{AppState, auth::Claims, jwt_manager::JwtManager};

#[derive(Debug)]
pub enum RoleError {
    MissingToken,
    InvalidToken,
    InsufficientPermissions,
}

impl IntoResponse for RoleError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RoleError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing or invalid token"),
            RoleError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            RoleError::InsufficientPermissions => (
                StatusCode::FORBIDDEN,
                "Insufficient permissions for this operation",
            ),
        };
        (status, error_message).into_response()
    }
}

pub trait RoleValidator: Send + Sync {
    fn validate(role: &str) -> bool;
    fn get_error() -> RoleError {
        RoleError::InsufficientPermissions
    }
}

pub struct AdminValidator;
impl RoleValidator for AdminValidator {
    fn validate(role: &str) -> bool {
        role == "admin"
    }
}

pub struct MemberValidator;
impl RoleValidator for MemberValidator {
    fn validate(role: &str) -> bool {
        role == "member" || role == "admin"
    }
}

pub struct LogSmartAdminValidator;
impl RoleValidator for LogSmartAdminValidator {
    fn validate(role: &str) -> bool {
        role == "logsmart_admin"
    }
}

#[derive(Debug)]
pub struct AuthorizedUser<T: RoleValidator>(pub Claims, std::marker::PhantomData<T>);

impl<T: 'static + RoleValidator> FromRequestParts<AppState> for AuthorizedUser<T> {
    type Rejection = RoleError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jwt_config = JwtManager::get_config();

        let TypedHeader(Authorization::<Bearer>(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                .await
                .map_err(|_| RoleError::MissingToken)?;

        let token = bearer.token();
        let claims = jwt_config
            .validate_token(token)
            .map_err(|_| RoleError::InvalidToken)?;

        let user = crate::db::get_user_by_id(&state.sqlite, &claims.user_id)
            .await
            .map_err(|_| RoleError::InvalidToken)?
            .ok_or(RoleError::InvalidToken)?;

        if T::validate(&user.role) {
            Ok(AuthorizedUser(claims, std::marker::PhantomData))
        } else {
            Err(T::get_error())
        }
    }
}

pub type AdminUser<T = AdminValidator> = AuthorizedUser<T>;
pub type MemberUser<T = MemberValidator> = AuthorizedUser<T>;
pub type LogSmartAdminUser<T = LogSmartAdminValidator> = AuthorizedUser<T>;
