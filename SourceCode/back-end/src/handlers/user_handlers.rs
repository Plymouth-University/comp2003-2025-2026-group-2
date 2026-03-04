use crate::{
    AppState, db,
    dto::{
        AdminUpdateMemberRequest, ErrorResponse, GetCompanyMembersResponse, RemoveMemberRequest,
    },
    logs_db,
    middleware::{AnyAuthUser, BranchManagerUser, ReadBranchUser},
    services::user_service::UserService,
    utils::{AuditLogger, err_bad_request, err_forbidden, err_internal},
};
use axum::{
    body::Bytes,
    extract::State,
    http::{header, StatusCode},
    Json,
};
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
    ReadBranchUser(_claims, user): ReadBranchUser,
    State(state): State<AppState>,
) -> Result<Json<GetCompanyMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("get_company_members called for user_id: {}", user.id);

    let members = db::get_company_members_for_user(&state.postgres, &user.id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company members: {:?}", e);
            err_internal("Database error")
        })?;

    if members.is_empty() {
        return Err(err_forbidden("User is not associated with a company"));
    }

    let filtered_members = if user.can_manage_company() || user.is_readonly_hq() {
        members
    } else if user.is_branch_manager() {
        members
            .into_iter()
            .filter(|m| m.branch_id == user.branch_id)
            .collect::<Vec<_>>()
    } else {
        members.into_iter().filter(|m| m.id == user.id).collect()
    };

    Ok(Json(filtered_members.into()))
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
    BranchManagerUser(_claims, user): BranchManagerUser,
    State(state): State<AppState>,
    Json(payload): Json<AdminUpdateMemberRequest>,
) -> Result<Json<GetCompanyMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
    if payload.first_name.is_empty() || payload.last_name.is_empty() {
        return Err(err_bad_request("First name and last name cannot be empty"));
    }

    let role = match payload.role.as_str() {
        "company_manager" => db::UserRole::CompanyManager,
        "branch_manager" => db::UserRole::BranchManager,
        "staff" => db::UserRole::Staff,
        _ => {
            return Err(err_bad_request(
                "Invalid role. Must be 'company_manager', 'branch_manager', or 'staff'",
            ));
        }
    };

    let updated_user = UserService::admin_update_member_profile(
        &state.postgres,
        &user,
        &payload.email,
        payload.first_name.clone(),
        payload.last_name.clone(),
        role,
        payload.branch_id,
        payload.profile_picture_id,
    )
    .await
    .map_err(|(status, error)| (status, Json(error)))?;

    // Invalidate cache for the updated user
    state.user_cache.invalidate(&updated_user.id).await;

    AuditLogger::log_admin_action(
        &state.postgres,
        user.id,
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
    BranchManagerUser(_claims, user): BranchManagerUser,
    State(state): State<AppState>,
    Json(payload): Json<RemoveMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let deleted_user_id = UserService::admin_delete_member(&state.postgres, &user, &payload.email)
        .await
        .map_err(|(status, error)| (status, Json(error)))?;

    // Invalidate cache for the deleted user
    state.user_cache.invalidate(&deleted_user_id).await;

    AuditLogger::log_admin_action(
        &state.postgres,
        user.id,
        format!("Deleted member: {}", payload.email),
    )
    .await;

    Ok(Json(json!({ "message": "Member deleted successfully" })))
}

#[utoipa::path(
    post,
    path = "/auth/profile-picture",
    request_body = Vec<u8>,
    responses(
        (status = 200, description = "Profile picture uploaded successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "User"
)]
pub async fn upload_profile_picture(
    AnyAuthUser(_claims, user): AnyAuthUser,
    State(state): State<AppState>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let data = body.to_vec();

    if data.len() > 10 * 1024 * 1024 {
        return Err(err_bad_request("File too large. Maximum size is 10MB"));
    }

    if data.is_empty() {
        return Err(err_bad_request("No file provided"));
    }

    let content_type = infer_content_type(&data);
    if !content_type.starts_with("image/") {
        return Err(err_bad_request("File must be an image"));
    }

    let file_id = logs_db::upload_profile_picture(&state.mongodb, data, &content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload profile picture: {:?}", e);
            err_internal("Failed to upload profile picture")
        })?;

    if let Some(old_picture_id) = &user.profile_picture_id {
        let _ = logs_db::delete_profile_picture(&state.mongodb, old_picture_id).await;
    }

    db::update_user_profile_picture_id(&state.postgres, &user.id, Some(&file_id))
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user profile picture: {:?}", e);
            err_internal("Failed to update profile picture reference")
        })?;

    state.user_cache.invalidate(&user.id).await;

    Ok(Json(json!({ "profile_picture_id": file_id, "profile_picture_url": format!("/api/auth/profile-picture/{}", file_id) })))
}

fn infer_content_type(data: &[u8]) -> String {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png".to_string()
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".to_string()
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

#[utoipa::path(
    get,
    path = "/auth/profile-picture/{file_id}",
    responses(
        (status = 200, description = "Profile picture", content_type = "image/webp"),
        (status = 404, description = "Picture not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "User"
)]
pub async fn get_profile_picture(
    State(state): State<AppState>,
    axum::extract::Path(file_id): axum::extract::Path<String>,
) -> Result<(
    StatusCode,
    [(header::HeaderName, header::HeaderValue); 1],
    Vec<u8>,
), (StatusCode, Json<serde_json::Value>)> {
    if let Some((content_type, data)) = logs_db::get_profile_picture(&state.mongodb, &file_id)
        .await
        .map_err(|e| err_internal(&e.to_string()))?
    {
        let header_value = header::HeaderValue::from_str(&content_type)
            .unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream"));
        return Ok((StatusCode::OK, [(header::CONTENT_TYPE, header_value)], data));
    }

    Err(err_bad_request("Profile picture not found"))
}

#[utoipa::path(
    delete,
    path = "/auth/profile-picture",
    responses(
        (status = 200, description = "Profile picture deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "User"
)]
pub async fn delete_profile_picture_handler(
    ReadBranchUser(_claims, user): ReadBranchUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Some(picture_id) = &user.profile_picture_id {
        let _ = logs_db::delete_profile_picture(&state.mongodb, picture_id).await;
    }

    db::update_user_profile_picture_id(&state.postgres, &user.id, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete profile picture: {:?}", e);
            err_internal("Failed to delete profile picture")
        })?;

    state.user_cache.invalidate(&user.id).await;

    Ok(Json(json!({ "message": "Profile picture deleted successfully" })))
}
