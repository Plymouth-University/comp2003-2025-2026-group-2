use crate::{
    AppState, db,
    dto::{BranchDto, CreateBranchRequest, ErrorResponse, ListBranchesResponse, UpdateBranchRequest},
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

    let branches = db::get_branches_by_company_id(&state.postgres, &company_id)
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
