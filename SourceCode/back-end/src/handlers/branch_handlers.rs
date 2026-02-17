use crate::{
    AppState,
    auth::generate_uuid6_token,
    db,
    dto::{
        BranchDto, ConfirmBranchDeletionRequest, ConfirmBranchDeletionResponse,
        CreateBranchRequest, ErrorResponse, ListBranchesResponse, RequestBranchDeletionRequest,
        RequestBranchDeletionResponse, UpdateBranchRequest,
    },
    email,
    middleware::AuthToken,
};
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

#[utoipa::path(
    post,
    path = "/auth/company/branches",
    request_body = CreateBranchRequest,
    responses(
        (status = 201, description = "Branch created successfully", body = BranchDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - only company manager can create branches", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Creates a new branch for the user's company.
pub async fn create_branch(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<CreateBranchRequest>,
) -> Result<(StatusCode, Json<BranchDto>), (StatusCode, Json<serde_json::Value>)> {
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

    if !user.is_company_manager() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company managers can create branches" })),
        ));
    }

    let company_id = user.company_id.ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let branch = db::create_branch(&state.postgres, company_id, payload.name, payload.address)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create branch: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to create branch" })),
            )
        })?;

    Ok((StatusCode::CREATED, Json(branch.into())))
}

#[utoipa::path(
    get,
    path = "/auth/company/branches",
    responses(
        (status = 200, description = "Branches retrieved successfully", body = ListBranchesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Lists all branches for the user's company.
pub async fn list_branches(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<ListBranchesResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let branches =
        db::get_branches_by_company_id_with_deletion_status(&state.postgres, &company_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error fetching branches: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database error" })),
                )
            })?;

    Ok(Json(ListBranchesResponse {
        branches: branches.into_iter().map(BranchDto::from).collect(),
    }))
}

#[utoipa::path(
    put,
    path = "/auth/company/branches",
    request_body = UpdateBranchRequest,
    responses(
        (status = 200, description = "Branch updated successfully", body = BranchDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - only company manager can update branches", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Updates an existing branch for the user's company.
pub async fn update_branch(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<UpdateBranchRequest>,
) -> Result<Json<BranchDto>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.is_company_manager() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company managers can update branches" })),
        ));
    }

    let company_id = user.company_id.ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    // Verify the branch belongs to the user's company
    let branch = db::get_branch_by_id(&state.postgres, &payload.branch_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching branch: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Branch not found" })),
        ))?;

    if branch.company_id != company_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Branch does not belong to your company" })),
        ));
    }

    let updated_branch = db::update_branch(
        &state.postgres,
        &payload.branch_id,
        &payload.name,
        &payload.address,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update branch: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to update branch" })),
        )
    })?;

    Ok(Json(updated_branch.into()))
}

#[utoipa::path(
    post,
    path = "/auth/company/branches/request-deletion",
    request_body = RequestBranchDeletionRequest,
    responses(
        (status = 200, description = "Branch deletion confirmation email sent", body = RequestBranchDeletionResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - only company manager can request branch deletion", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company Management"
)]
/// Requests branch deletion by sending a confirmation email.
pub async fn request_branch_deletion(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<RequestBranchDeletionRequest>,
) -> Result<Json<RequestBranchDeletionResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let company_id = user.company_id.as_ref().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    if !user.can_manage_company() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only company managers can delete branches" })),
        ));
    }

    let branch = db::get_branch_by_id(&state.postgres, &payload.branch_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching branch: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Branch not found" })),
        ))?;

    if &branch.company_id != company_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Branch does not belong to your company" })),
        ));
    }

    let token = generate_uuid6_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    db::create_branch_deletion_token(
        &state.postgres,
        user.id.clone(),
        payload.branch_id,
        token.clone(),
        expires_at,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create branch deletion token: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to process deletion request" })),
        )
    })?;

    let confirmation_link = format!(
        "{}/confirm-branch-deletion?token={}",
        "https://logsmart.app", token
    );

    email::send_branch_deletion_confirmation_email(&user.email, &branch.name, &confirmation_link)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send branch deletion confirmation email: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to send confirmation email" })),
            )
        })?;

    Ok(Json(RequestBranchDeletionResponse {
        message: "A confirmation email has been sent to your email address. Please check your inbox to complete the branch deletion.".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/company/branches/confirm-deletion",
    request_body = ConfirmBranchDeletionRequest,
    responses(
        (status = 200, description = "Branch deleted successfully", body = ConfirmBranchDeletionResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Branch not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Company Management"
)]
/// Confirms and executes branch deletion using a token.
pub async fn confirm_branch_deletion(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmBranchDeletionRequest>,
) -> Result<Json<ConfirmBranchDeletionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let token_record = db::get_branch_deletion_token(&state.postgres, &payload.token)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching deletion token: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    let (token_id, user_id, branch_id) = token_record.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Invalid or expired confirmation token" })),
    ))?;

    let branch = db::get_branch_by_id(&state.postgres, &branch_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching branch: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            )
        })?;

    let branch_name = match &branch {
        Some(b) => b.name.clone(),
        None => "Unknown".to_string(),
    };

    db::delete_branch(&state.postgres, &branch_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete branch: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete branch" })),
            )
        })?;

    db::mark_branch_deletion_token_used(&state.postgres, &token_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark deletion token as used: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to complete deletion" })),
            )
        })?;

    let user = db::get_user_by_id(&state.postgres, &user_id)
        .await
        .ok()
        .flatten();

    if let Some(user) = user {
        let _ = email::send_branch_deleted_notification_email(&user.email, &branch_name).await;
    }

    Ok(Json(ConfirmBranchDeletionResponse {
        message: format!("Branch '{}' has been successfully deleted.", branch_name),
    }))
}
