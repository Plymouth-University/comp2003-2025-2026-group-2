use crate::{
    AppState, db,
    dto::{
        AddTemplateRequest, AddTemplateResponse, DeleteTemplateRequest, DeleteTemplateResponse,
        ErrorResponse, GetAllTemplatesResponse, GetTemplateRequest, GetTemplateResponse,
        GetTemplateVersionsResponse, RenameTemplateRequest, RenameTemplateResponse,
        RestoreTemplateVersionRequest, TemplateInfo, UpdateTemplateRequest, UpdateTemplateResponse,
    },
    middleware::AuthToken,
    services,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde_json::json;

#[utoipa::path(
    post,
    path = "/logs/templates",
    request_body = AddTemplateRequest,
    responses(
        (status = 200, description = "Template added successfully", body = AddTemplateResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Adds a new log template for the current company.
pub async fn add_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<AddTemplateRequest>,
) -> Result<Json<AddTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_manage_branch() || user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can create templates" })),
        ));
    }

    // Branch managers can only create templates for their own branch
    if user.is_branch_manager() {
        if payload.branch_id.is_none() {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "Branch managers cannot create company-wide templates" })),
            ));
        }
        if payload.branch_id != user.branch_id {
            return Err((
                StatusCode::FORBIDDEN,
                Json(
                    json!({ "error": "Branch managers can only create templates for their own branch" }),
                ),
            ));
        }
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    services::TemplateService::create_template(
        &state,
        &company_id,
        payload.template_name,
        payload.template_layout,
        payload.schedule,
        &claims.user_id,
        payload.branch_id,
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(AddTemplateResponse {
        message: "Template added successfully.".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/logs/templates",
    params(
        ("template_name", description = "Name of the template to retrieve", example = "ErrorLog" )
    ),
    responses(
        (status = 200, description = "Template retrieved successfully", body = GetTemplateResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Retrieves a specific log template by name.
pub async fn get_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(payload): Query<GetTemplateRequest>,
) -> Result<Json<GetTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let (template_name, template_layout, version, version_name) =
        services::TemplateService::get_template(&state, &company_id, &payload.template_name)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(GetTemplateResponse {
        template_name,
        template_layout,
        version,
        version_name,
    }))
}

#[utoipa::path(
    get,
    path = "/logs/templates/all",
    responses(
        (status = 200, description = "All templates retrieved successfully", body = GetAllTemplatesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Retrieves all log templates for the current company/branch.
pub async fn get_all_templates(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<GetAllTemplatesResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_read_manage_branch() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can access templates" })),
        ));
    }

    let company_id = user.company_id.as_deref().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    let templates = services::TemplateService::get_all_templates(
        &state,
        company_id,
        if user.is_branch_manager() {
            user.branch_id.as_deref()
        } else {
            None
        },
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    let response_templates = templates
        .into_iter()
        .map(
            |(name, created_at, updated_at, user_id, schedule)| TemplateInfo {
                template_name: name,
                created_at: created_at.to_string(),
                updated_at: updated_at.to_string(),
                created_by: user_id,
                schedule,
            },
        )
        .collect();

    Ok(Json(GetAllTemplatesResponse {
        templates: response_templates,
    }))
}

#[utoipa::path(
    put,
    path = "/logs/templates/update",
    request_body = UpdateTemplateRequest,
    responses(
        (status = 200, description = "Template updated successfully", body = UpdateTemplateResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Updates an existing log template.
pub async fn update_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<UpdateTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_manage_branch() || user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can update templates" })),
        ));
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    services::TemplateService::update_template(
        &state,
        &company_id,
        &payload.template_name,
        payload.template_layout.as_ref(),
        payload.schedule.as_ref(),
        &claims.user_id,
        payload.version_name.clone(),
        user.branch_id.as_deref(),
        user.is_company_manager(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;
    Ok(Json(UpdateTemplateResponse {
        message: "Template updated successfully.".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/logs/templates/versions",
    params(
        ("template_name", description = "Name of the template to retrieve versions for", example = "ErrorLog")
    ),
    responses(
        (status = 200, description = "Versions retrieved successfully", body = GetTemplateVersionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Retrieves the version history of a log template.
pub async fn get_template_versions(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(payload): Query<GetTemplateRequest>,
) -> Result<Json<crate::dto::GetTemplateVersionsResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let versions =
        services::TemplateService::get_versions(&state, &company_id, &payload.template_name)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    let version_infos = versions
        .into_iter()
        .map(|v| crate::dto::TemplateVersionInfo {
            version: v.version,
            version_name: v.version_name,
            created_at: v.created_at.to_string(),
            created_by: v.created_by.to_string(),
        })
        .collect();

    Ok(Json(crate::dto::GetTemplateVersionsResponse {
        versions: version_infos,
    }))
}

#[utoipa::path(
    post,
    path = "/logs/templates/versions/restore",
    params(
        ("template_name", description = "Name of the template to restore", example = "ErrorLog")
    ),
    request_body = RestoreTemplateVersionRequest,
    responses(
        (status = 200, description = "Template restored successfully", body = UpdateTemplateResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Restores a specific version of a log template.
pub async fn restore_template_version(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(query): Query<GetTemplateRequest>,
    Json(payload): Json<crate::dto::RestoreTemplateVersionRequest>,
) -> Result<Json<UpdateTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_manage_branch() || user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can restore templates" })),
        ));
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    services::TemplateService::restore_version(
        &state,
        &company_id,
        &query.template_name,
        payload.version,
        &claims.user_id,
        user.branch_id.as_deref(),
        user.is_company_manager(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(UpdateTemplateResponse {
        message: format!("Template restored to version {}", payload.version),
    }))
}

#[utoipa::path(
    put,
    path = "/logs/templates/rename",
    request_body = RenameTemplateRequest,
    responses(
        (status = 200, description = "Template renamed successfully", body = RenameTemplateResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Renames an existing log template.
pub async fn rename_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<RenameTemplateRequest>,
) -> Result<Json<RenameTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_manage_branch() || user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can rename templates" })),
        ));
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    services::TemplateService::rename_template(
        &state,
        &company_id,
        &payload.old_template_name,
        &payload.new_template_name,
        user.branch_id.as_deref(),
        user.is_company_manager(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;
    Ok(Json(RenameTemplateResponse {
        message: "Template renamed successfully.".to_string(),
    }))
}

#[utoipa::path(
    delete,
    path = "/logs/templates",
    params(
        DeleteTemplateRequest
    ),
    responses(
        (status = 200, description = "Template deleted successfully", body = DeleteTemplateResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
/// Deletes a specific log template.
pub async fn delete_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(payload): Query<DeleteTemplateRequest>,
) -> Result<Json<DeleteTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    if !user.can_manage_branch() || user.is_readonly_hq() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only managers can delete templates" })),
        ));
    }

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "User is not associated with a company" })),
    ))?;

    services::TemplateService::delete_template(
        &state,
        &company_id,
        &payload.template_name,
        user.branch_id.as_deref(),
        user.is_company_manager(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;
    Ok(Json(DeleteTemplateResponse {
        message: "Template deleted successfully.".to_string(),
    }))
}
