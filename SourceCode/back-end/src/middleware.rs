use crate::{AppState, auth::Claims, db::UserRecord, jwt_manager::JwtManager, utils::AuditContext};
use axum::{
    Json,
    extract::{ConnectInfo, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use serde_json::json;
use std::convert::Infallible;

async fn extract_claims(parts: &mut Parts) -> Result<Claims, AuthError> {
    let jwt_config = JwtManager::get_config();

    let TypedHeader(Authorization::<Bearer>(bearer)) =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
            .await
            .map_err(|_| AuthError::MissingToken)?;

    let token = bearer.token();
    jwt_config
        .validate_token(token)
        .map_err(|_| AuthError::InvalidToken)
}

async fn get_authenticated_user(
    state: &AppState,
    claims: &Claims,
) -> Result<UserRecord, RoleError> {
    let user_id = &claims.user_id;
    if let Some(user) = state.user_cache.get(user_id).await {
        if user.company_deleted_at.is_some() {
            return Err(RoleError::CompanyDeleted(
                "Your company has been deleted. Please contact support.".to_string(),
            ));
        }
        return Ok(user);
    }

    let user = crate::db::get_user_by_id(&state.postgres, user_id)
        .await
        .map_err(|_| RoleError::InvalidToken)?
        .ok_or(RoleError::InvalidToken)?;

    if user.company_deleted_at.is_some() {
        return Err(RoleError::CompanyDeleted(
            "Your company has been deleted. Please contact support.".to_string(),
        ));
    }

    state.user_cache.insert(user_id.clone(), user.clone()).await;
    Ok(user)
}

pub struct AuthToken(pub Claims);

pub struct AuditRequestContext(pub AuditContext);

impl FromRequestParts<crate::AppState> for AuditRequestContext {
    type Rejection = Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let addr = parts
                .extensions
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|c| &c.0);

            let ctx = AuditContext {
                ip_address: crate::utils::extract_optional_ip_from_headers_and_addr(
                    &parts.headers,
                    addr,
                ),
                user_agent: crate::utils::extract_user_agent(&parts.headers),
                request_path: Some(parts.uri.path().to_string()),
                request_method: Some(parts.method.to_string()),
                ..AuditContext::default()
            };

            Ok(AuditRequestContext(ctx))
        })
    }
}

impl FromRequestParts<crate::AppState> for AuthToken {
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &crate::AppState,
    ) -> impl std::future::Future<
        Output = Result<Self, <Self as FromRequestParts<AppState>>::Rejection>,
    > + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;
            Ok(AuthToken(claims))
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing authorization token",
            ),
            AuthError::InvalidToken => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            ),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

#[derive(Debug)]
pub enum RoleError {
    MissingToken,
    InvalidToken,
    InsufficientPermissions,
    CompanyDeleted(String),
}

impl From<AuthError> for RoleError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::MissingToken => RoleError::MissingToken,
            AuthError::InvalidToken => RoleError::InvalidToken,
        }
    }
}

impl IntoResponse for RoleError {
    fn into_response(self) -> Response {
        match self {
            RoleError::MissingToken => {
                let body = Json(json!({ "error": "Missing authorization token" }));
                (axum::http::StatusCode::UNAUTHORIZED, body).into_response()
            }
            RoleError::InvalidToken => {
                let body = Json(json!({ "error": "Invalid or expired token" }));
                (axum::http::StatusCode::UNAUTHORIZED, body).into_response()
            }
            RoleError::InsufficientPermissions => {
                let body = Json(json!({ "error": "Insufficient permissions for this operation" }));
                (axum::http::StatusCode::FORBIDDEN, body).into_response()
            }
            RoleError::CompanyDeleted(msg) => {
                let body = Json(json!({ "error": msg }));
                (axum::http::StatusCode::FORBIDDEN, body).into_response()
            }
        }
    }
}

/// Extractor for `CompanyManager` and above
pub struct ManageCompanyUser(pub Claims, pub UserRecord);

impl FromRequestParts<crate::AppState> for ManageCompanyUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !user.can_manage_company() {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(ManageCompanyUser(claims, user))
        })
    }
}

/// Extractor for Staff and above
pub struct AnyAuthUser(pub Claims, pub UserRecord);

impl FromRequestParts<crate::AppState> for AnyAuthUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !matches!(
                user.get_role(),
                crate::db::UserRole::Staff
                    | crate::db::UserRole::CompanyManager
                    | crate::db::UserRole::BranchManager
                    | crate::db::UserRole::LogSmartAdmin
            ) {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(AnyAuthUser(claims, user))
        })
    }
}

/// Extractor for `BranchManager` and above
pub struct BranchManagerUser(pub Claims, pub UserRecord);
impl FromRequestParts<crate::AppState> for BranchManagerUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !user.can_manage_branch() {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(BranchManagerUser(claims, user))
        })
    }
}

/// Extractor for `LogSmartAdmin` only
pub struct LogSmartAdminUser(pub Claims, pub UserRecord);

impl FromRequestParts<crate::AppState> for LogSmartAdminUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !user.is_logsmart_admin() {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(LogSmartAdminUser(claims, user))
        })
    }
}

/// Extractor for company admin and hq staff
pub struct ReadCompanyUser(pub Claims, pub UserRecord);
impl FromRequestParts<crate::AppState> for ReadCompanyUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !(user.can_manage_company() || user.is_readonly_hq()) {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(ReadCompanyUser(claims, user))
        })
    }
}

/// Extractor for branch admin and hq staff
pub struct ReadBranchUser(pub Claims, pub UserRecord);
impl FromRequestParts<crate::AppState> for ReadBranchUser {
    type Rejection = RoleError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        Box::pin(async move {
            let claims = extract_claims(parts).await?;

            let user = get_authenticated_user(state, &claims).await?;

            if !(user.can_manage_branch() || user.is_readonly_hq()) {
                return Err(RoleError::InsufficientPermissions);
            }

            Ok(ReadBranchUser(claims, user))
        })
    }
}
