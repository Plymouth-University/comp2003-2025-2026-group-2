use crate::{
    AppState, db,
    dto::{
        AdminUpdateMemberRequest, ErrorResponse, GetCompanyMembersResponse, RemoveMemberRequest,
    },
    middleware::AuthToken,
    services::user_service::UserService,
    utils::AuditLogger,
};
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/auth/company/members",
    responses(
        (status = 200, description = "Company members retrieved successfully", body = [GetCompanyMembersResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Retrieves all members belonging to the current user's company.
///
/// # Errors
/// Returns an error if the user is not associated with a company or if the database query fails.
pub async fn get_company_members(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<GetCompanyMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let members = db::get_users_by_company_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company members: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    Ok(Json(members.into()))
}

#[utoipa::path(
    put,
    path = "/auth/admin/update-member",
    request_body = AdminUpdateMemberRequest,
    responses(
        (status = 200, description = "Member profile updated successfully", body = GetCompanyMembersResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - not an admin or different company", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Updates a company member's profile (admin only).
///
/// # Errors
/// Returns an error if the user is not an admin, the request is invalid, or the update fails.
pub async fn admin_update_member_profile(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<AdminUpdateMemberRequest>,
) -> Result<Json<GetCompanyMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.first_name.is_empty() || payload.last_name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "First name and last name cannot be empty" })),
        ));
    }

    let role = match payload.role.as_str() {
        "admin" => db::UserRole::Admin,
        "member" => db::UserRole::Member,
        "logsmart_admin" => db::UserRole::LogSmartAdmin,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid role. Must be 'admin', 'member', or 'logsmart_admin'" })),
            ));
        }
    };

    let updated_user = UserService::admin_update_member_profile(
        &state.postgres,
        &claims.user_id,
        &payload.email,
        payload.first_name.clone(),
        payload.last_name.clone(),
        role,
    )
    .await
    .map_err(|(status, error)| (status, Json(error)))?;

    AuditLogger::log_admin_action(
        &state.postgres,
        claims.user_id,
        format!("Updated member profile: {}", updated_user.email),
    )
    .await;

    Ok(Json(vec![updated_user].into()))
}
#[utoipa::path(
    delete,
    path = "/auth/admin/remove-member",
    request_body = RemoveMemberRequest,
    responses(
        (status = 200, description = "Member deleted successfully"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - not an admin or different company", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Deletes a company member (admin only).
///
/// # Errors
/// Returns an error if the user is not an admin or if the deletion fails.
pub async fn admin_delete_member(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<RemoveMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    UserService::admin_delete_member(&state.postgres, &claims.user_id, &payload.email)
        .await
        .map_err(|(status, error)| (status, Json(error)))?;

    AuditLogger::log_admin_action(
        &state.postgres,
        claims.user_id,
        format!("Deleted member: {}", payload.email),
    )
    .await;

    Ok(Json(json!({ "message": "Member deleted successfully" })))
}
