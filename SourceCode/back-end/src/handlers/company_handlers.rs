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
use serde_json::json;

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
