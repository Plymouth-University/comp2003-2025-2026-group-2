use crate::{
    AppState, db,
    dto::ErrorResponse,
    images_db,
    middleware::ManageCompanyUser,
    utils::{err_bad_request, err_forbidden, err_internal, err_not_found},
};
use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{StatusCode, header},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub logo_id: Option<String>,
    pub logo_url: Option<String>,
    pub data_exported_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<db::Company> for CompanyResponse {
    fn from(company: db::Company) -> Self {
        let logo_id = company.logo_id.clone();
        let logo_url = logo_id
            .clone()
            .map(|_id| format!("/api/companies/{}/logo", company.id));
        Self {
            id: company.id,
            name: company.name,
            address: company.address,
            logo_id,
            logo_url,
            data_exported_at: company.data_exported_at,
            deleted_at: company.deleted_at,
            deletion_requested_at: company.deletion_requested_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCompanyRequest {
    pub name: String,
    pub address: String,
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
    post,
    path = "/companies/{company_id}/logo",
    request_body = Vec<u8>,
    responses(
        (status = 200, description = "Logo uploaded successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn upload_company_logo(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only manage your own company's logo"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

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

    let file_id = images_db::upload_company_logo(&state.mongodb, data, &company_id, &content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload company logo: {:?}", e);
            err_internal("Failed to upload logo")
        })?;

    if let Err(err) = db::update_company_logo_id(&state.postgres, &company_id, Some(&file_id)).await
    {
        tracing::error!("Failed to update company logo: {:?}", err);
        if let Err(delete_err) = images_db::delete_company_logo(&state.mongodb, &file_id).await {
            tracing::error!("Failed to cleanup uploaded logo: {:?}", delete_err);
        }
        return Err(err_internal("Failed to update logo reference"));
    }

    if let Some(old_logo_id) = &company.logo_id
        && let Err(err) = images_db::delete_company_logo(&state.mongodb, old_logo_id).await
    {
        tracing::error!("Failed to delete old company logo: {:?}", err);
    }

    Ok(Json(json!({
        "logo_id": file_id,
        "logo_url": format!("/api/companies/{}/logo", company_id)
    })))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}/logo",
    responses(
        (status = 200, description = "Company logo", content_type = "image/webp"),
        (status = 404, description = "Logo not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn get_company_logo(
    ManageCompanyUser(_claims, _user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, header::HeaderValue); 1],
        Vec<u8>,
    ),
    (StatusCode, Json<serde_json::Value>),
> {
    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let logo_id = company
        .logo_id
        .ok_or_else(|| err_not_found("Logo not found"))?;

    if let Some((content_type, data)) = images_db::get_company_logo(&state.mongodb, &logo_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get company logo: {:?}", e);
            err_internal("Failed to get logo")
        })?
    {
        let content_type_header = header::HeaderValue::from_str(&content_type)
            .unwrap_or(header::HeaderValue::from_static("application/octet-stream"));
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type_header)],
            data,
        ))
    } else {
        Err(err_not_found("Logo not found"))
    }
}

#[utoipa::path(
    delete,
    path = "/companies/{company_id}/logo",
    responses(
        (status = 200, description = "Logo deleted successfully", body = serde_json::Value),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn delete_company_logo(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only manage your own company's logo"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let Some(logo_id) = company.logo_id else {
        return Err(err_not_found("No logo to delete"));
    };

    db::update_company_logo_id(&state.postgres, &company_id, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update company logo: {:?}", e);
            err_internal("Failed to delete logo reference")
        })?;

    if let Err(err) = images_db::delete_company_logo(&state.mongodb, &logo_id).await {
        tracing::error!("Failed to delete company logo from storage: {:?}", err);
        let _ = db::update_company_logo_id(&state.postgres, &company_id, Some(&logo_id)).await;
        return Err(err_internal("Failed to delete logo"));
    }

    Ok(Json(json!({ "message": "Logo deleted successfully" })))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}",
    responses(
        (status = 200, description = "Company details", body = CompanyResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn get_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only view your own company"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    Ok(Json(CompanyResponse::from(company)))
}

#[utoipa::path(
    put,
    path = "/companies/{company_id}",
    request_body = UpdateCompanyRequest,
    responses(
        (status = 200, description = "Company updated successfully", body = CompanyResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn update_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only update your own company"));
    }

    if payload.name.trim().is_empty() {
        return Err(err_bad_request("Company name cannot be empty"));
    }

    if payload.address.trim().is_empty() {
        return Err(err_bad_request("Company address cannot be empty"));
    }

    let company = db::update_company(
        &state.postgres,
        &company_id,
        &payload.name,
        &payload.address,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update company: {:?}", e);
        err_internal("Failed to update company")
    })?;

    // Note: Possible performance issue
    if let Ok(users) = db::get_users_by_company_id(&state.postgres, &company_id).await {
        for user in users {
            state.user_cache.invalidate(&user.id).await;
        }
    }

    Ok(Json(CompanyResponse::from(company)))
}

#[utoipa::path(
    post,
    path = "/companies/{company_id}/export",
    responses(
        (status = 200, description = "Data export initiated", body = serde_json::Value),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn export_company_data(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only export your own company data"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    let company_email = user.email.clone();

    let company_name = company.name.clone();
    let company_address = company.address.clone();

    tokio::spawn(async move {
        if let Err(e) =
            crate::email::send_company_data_export(&company_email, &company_name, &company_address)
                .await
        {
            tracing::error!("Failed to send company data export email: {:?}", e);
        }
    });

    if let Err(e) = db::mark_company_data_exported(&state.postgres, &company_id).await {
        tracing::error!("Failed to mark company data as exported: {:?}", e);
    }

    Ok(Json(
        json!({ "message": "Data export initiated. You will receive an email shortly." }),
    ))
}

#[utoipa::path(
    delete,
    path = "/companies/{company_id}",
    responses(
        (status = 200, description = "Company deletion requested", body = serde_json::Value),
        (status = 400, description = "Data must be exported first", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "Company"
)]
pub async fn delete_company(
    ManageCompanyUser(_claims, user): ManageCompanyUser,
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if user.company_id.as_ref() != Some(&company_id) {
        return Err(err_forbidden("You can only delete your own company"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if company.data_exported_at.is_none() {
        return Err(err_bad_request(
            "You must export company data before requesting deletion",
        ));
    }

    let company_email = user.email.clone();
    let company_name = company.name.clone();

    let updated_company = db::request_company_deletion(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to request company deletion: {:?}", e);
            err_internal("Failed to request deletion")
        })?;

    let deletion_token = updated_company.deletion_token.unwrap_or_default();

    tokio::spawn(async move {
        if let Err(e) = crate::email::send_company_deletion_request(
            &company_email,
            &company_name,
            &company_id,
            &deletion_token,
        )
        .await
        {
            tracing::error!(
                "Failed to send company deletion request email: {:?}",
                e
            );
        }
    });

    Ok(Json(
        json!({ "message": "Deletion requested. Please check your email to confirm." }),
    ))
}

#[utoipa::path(
    get,
    path = "/companies/{company_id}/confirm-deletion",
    responses(
        (status = 200, description = "Company deletion confirmed", body = serde_json::Value),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Company not found", body = ErrorResponse),
    ),
    tag = "Company"
)]
pub async fn confirm_company_deletion(
    State(state): State<AppState>,
    axum::extract::Path(company_id): axum::extract::Path<String>,
    axum::extract::Query(token): axum::extract::Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token = token.get("token").and_then(|t| t.as_str()).unwrap_or("");

    if token.is_empty() {
        return Err(err_bad_request("Confirmation token is required"));
    }

    let company = db::get_company_by_id(&state.postgres, &company_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching company: {:?}", e);
            err_internal("Database error")
        })?
        .ok_or_else(|| err_not_found("Company not found"))?;

    if company.deletion_token.as_deref() != Some(token) {
        return Err(err_bad_request("Invalid or expired confirmation token"));
    }

    let company_email = company.name.clone();

    let _company = db::confirm_company_deletion(&state.postgres, &company_id, token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to confirm company deletion: {:?}", e);
            err_internal("Failed to confirm deletion")
        })?;

    tokio::spawn(async move {
        if let Err(e) = crate::email::send_company_deleted_notification(&company_email).await {
            tracing::error!(
                "Failed to send company deletion notification: {:?}",
                e
            );
        }
    });

    Ok(Json(
        json!({ "message": "Company has been deleted. Data will be retained for 30 days." }),
    ))
}
