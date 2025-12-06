use crate::{
    AppState, db,
    dto::{
        AddTemplateRequest, AddTemplateResponse, DeleteTemplateRequest, DeleteTemplateResponse,
        ErrorResponse, GetAllTemplatesResponse, GetTemplateRequest, GetTemplateResponse,
        RenameTemplateRequest, RenameTemplateResponse, TemplateInfo, UpdateTemplateRequest,
        UpdateTemplateResponse,
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
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn add_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<AddTemplateRequest>,
) -> Result<Json<AddTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
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
            Json(json!({ "error": "Only company admin or LogSmartAdmin can create templates" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    services::TemplateService::create_template(
        &state,
        &company_id,
        payload.template_name,
        payload.template_layout,
        payload.schedule,
        &claims.user_id,
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
        (status = 200, description = "Password reset successfully", body = GetTemplateResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn get_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(payload): Query<GetTemplateRequest>,
) -> Result<Json<GetTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    let (template_name, template_layout) =
        services::TemplateService::get_template(&state, &company_id, &payload.template_name)
            .await
            .map_err(|(status, err)| (status, Json(err)))?;

    Ok(Json(GetTemplateResponse {
        template_name,
        template_layout,
    }))
}

#[utoipa::path(
    get,
    path = "/logs/templates/all",
    responses(
        (status = 200, description = "All templates retrieved successfully", body = GetAllTemplatesResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn get_all_templates(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
) -> Result<Json<GetAllTemplatesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
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
            Json(json!({ "error": "Only company admin or LogSmartAdmin can access templates" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    let templates = services::TemplateService::get_all_templates(&state, &company_id)
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
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn update_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<UpdateTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
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
            Json(json!({ "error": "Only company admin or LogSmartAdmin can update templates" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    services::TemplateService::update_template(
        &state,
        &company_id,
        &payload.template_name,
        payload.template_layout.as_ref(),
        payload.schedule.as_ref(),
    )
    .await
    .map_err(|(status, err)| (status, Json(err)))?;
    Ok(Json(UpdateTemplateResponse {
        message: "Template updated successfully.".to_string(),
    }))
}

#[utoipa::path(
    put,
    path = "/logs/templates/rename",
    request_body = RenameTemplateRequest,
    responses(
        (status = 200, description = "Template renamed successfully", body = RenameTemplateResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn rename_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Json(payload): Json<RenameTemplateRequest>,
) -> Result<Json<RenameTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
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
            Json(json!({ "error": "Only company admin or LogSmartAdmin can rename templates" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    services::TemplateService::rename_template(
        &state,
        &company_id,
        &payload.old_template_name,
        &payload.new_template_name,
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
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 400, description = "Password validation failed", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Templates"
)]
pub async fn delete_template(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    Query(payload): Query<DeleteTemplateRequest>,
) -> Result<Json<DeleteTemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = db::get_user_by_id(&state.sqlite, &claims.user_id)
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
            Json(json!({ "error": "Only company admin or LogSmartAdmin can delete templates" })),
        ));
    }

    let company_id = db::get_user_company_id(&state.sqlite, &claims.user_id)
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

    services::TemplateService::delete_template(&state, &company_id, &payload.template_name)
        .await
        .map_err(|(status, err)| (status, Json(err)))?;
    Ok(Json(DeleteTemplateResponse {
        message: "Template deleted successfully.".to_string(),
    }))
}
