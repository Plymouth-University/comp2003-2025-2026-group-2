use crate::dto::GetPendingInvitationsResponse;
use crate::utils::{extract_ip_from_headers_and_addr, extract_user_agent};
use crate::{
    AppState,
    auth::{hash_password, validate_email, validate_password_policy},
    db,
    dto::{
        AcceptInvitationRequest, AuthResponse, CancelInvitationRequest, ErrorResponse,
        GetInvitationDetailsRequest, GetInvitationDetailsResponse, InvitationResponse,
        InviteUserRequest, UserResponse,
    },
    jwt_manager::JwtManager,
    middleware::AuthToken,
    services,
    utils::AuditLogger,
};
use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
    http::{self, HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use serde_json::json;

#[utoipa::path(
    post,
    path = "/auth/invitations/send",
    request_body = InviteUserRequest,
    responses(
        (status = 201, description = "Invitation sent successfully", body = InvitationResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - invalid or missing token", body = ErrorResponse),
        (status = 409, description = "User already invited or exists", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn invite_user(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<InviteUserRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Email is required" })),
        ));
    }

    if let Err(e) = validate_email(&payload.email) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    let user = db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        ))?;

    if !user.can_manage_company() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company admin or LogSmartAdmin can invite users" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user company ID: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User is not associated with a company" })),
        ))?;

    let (invitation_id, expires_at) = services::InvitationService::send_invitation(
        &state.postgres,
        claims.user_id,
        user.email,
        payload.email.clone(),
        company_id,
        Some(ip_address),
        user_agent,
    )
    .await
    .map_err(|(status, err)| {
        state.metrics.increment_failed_requests();
        (status, Json(err))
    })?;

    state.metrics.increment_invitations_sent();
    state.metrics.increment_successful_requests();

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            id: invitation_id,
            email: payload.email,
            expires_at,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/invitations/accept",
    request_body = AcceptInvitationRequest,
    responses(
        (status = 201, description = "Invitation accepted successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Invalid or expired invitation token", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Invitations"
)]
pub async fn accept_invitation(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let _timer = crate::metrics::RequestTimer::new();
    state.metrics.increment_total_requests();

    let ip_address = extract_ip_from_headers_and_addr(&headers, &addr);
    let user_agent = extract_user_agent(&headers);

    if payload.token.is_empty()
        || payload.first_name.is_empty()
        || payload.last_name.is_empty()
        || payload.password.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields" })),
        ));
    }

    if let Err(e) = validate_password_policy(&payload.password) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ));
    }

    let invitation = db::get_invitation_by_token(&state.postgres, &payload.token)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching invitation by token: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired invitation" })),
        ))?;

    let now = chrono::Utc::now();

    if now > invitation.expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invitation has expired" })),
        ));
    }

    if db::get_user_by_email(&state.postgres, &invitation.email)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error checking existing user during invitation acceptance: {:?}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "Email already has an account" })),
        ));
    }

    let password_hash = hash_password(&payload.password).map_err(|e| {
        tracing::error!(
            "Failed to hash password during invitation acceptance: {:?}",
            e
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to process password" })),
        )
    })?;

    let _created_user = db::accept_invitation_with_user_creation(
        &state.postgres,
        &invitation.id,
        &invitation.email,
        payload.first_name.clone(),
        payload.last_name.clone(),
        password_hash,
        &invitation.company_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user and accept invitation: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create user" })),
        )
    })?;

    let user_id = _created_user.id.clone();

    let jwt_config = JwtManager::get_config();
    let token = jwt_config
        .generate_token(user_id.clone(), 24)
        .map_err(|e| {
            tracing::error!(
                "Failed to generate JWT token for invitation acceptance: {:?}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate token" })),
            )
        })?;

    AuditLogger::log_invitation_accepted(
        &state.postgres,
        user_id.clone(),
        invitation.email.clone(),
        invitation.company_id.clone(),
        Some(ip_address),
        user_agent,
    )
    .await;

    state.metrics.increment_invitations_accepted();
    state.metrics.increment_successful_requests();
    tracing::info!("Invitation accepted by user: {}", user_id);

    let cookie = format!(
        "ls-token={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
        token,
        60 * 60 * 24 * 7
    );

    let mut response = (
        StatusCode::CREATED,
        Json(AuthResponse {
            token: token.clone(),
            user: UserResponse {
                email: invitation.email,
                first_name: payload.first_name,
                last_name: payload.last_name,
                company_name: None,
                role: _created_user.role,
                oauth_provider: None,
            },
        }),
    )
        .into_response();

    response.headers_mut().insert(
        SET_COOKIE,
        http::header::HeaderValue::from_str(&cookie).map_err(|e| {
            tracing::error!(
                "Failed to set cookie in invitation acceptance response: {:?}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to set authentication cookie" })),
            )
        })?,
    );

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/auth/invitations/details",
    params(
        GetInvitationDetailsRequest
    ),
    responses(
        (status = 200, description = "Invitation details retrieved successfully", body = GetInvitationDetailsResponse),
        (status = 404, description = "Invitation not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Invitations"
)]
pub async fn get_invitation_details(
    State(state): State<AppState>,
    Query(payload): Query<GetInvitationDetailsRequest>,
) -> Result<Json<GetInvitationDetailsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let invitation = db::get_invitation_by_token(&state.postgres, &payload.token)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching invitation by token: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Invitation not found" })),
        ))?;

    let company_name = db::get_company_by_id(&state.postgres, &invitation.company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company name: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Company not found" })),
        ))?
        .name;

    Ok(Json(GetInvitationDetailsResponse {
        company_name,
        expires_at: invitation.expires_at,
    }))
}

#[utoipa::path(
    get,
    path = "/auth/invitations/pending",
    responses(
        (status = 200, description = "Invitations retrieved successfully", body = [InvitationResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Invitations"
)]
pub async fn get_pending_invitations(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<GetPendingInvitationsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        ))?;

    if !user.can_manage_company() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company admin or LogSmartAdmin can view invitations" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user company ID: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "User is not associated with a company" })),
        ))?;

    let invitations =
        services::InvitationService::get_pending_invitations(&state.postgres, &company_id)
            .await
            .map(|inv_list| {
                inv_list
                    .into_iter()
                    .map(|inv| InvitationResponse {
                        id: inv.id,
                        email: inv.email,
                        expires_at: inv.expires_at,
                    })
                    .collect()
            })
            .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(GetPendingInvitationsResponse { invitations }))
}
#[utoipa::path(
    put,
    path = "/auth/invitations/cancel",
    request_body = CancelInvitationRequest,
    responses(
        (status = 200, description = "Invitation cancelled successfully"),
        (status = 400, description = "Invalid request or invitation already accepted/cancelled", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - not an admin or different company", body = ErrorResponse),
        (status = 404, description = "Invitation not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Invitations"
)]
pub async fn cancel_invitation(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<crate::dto::CancelInvitationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    services::InvitationService::cancel_invitation(
        &state.postgres,
        &claims.user_id,
        &payload.invitation_id,
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(
        json!({ "message": "Invitation cancelled successfully" }),
    ))
}
