use crate::AppState;
use crate::dto::LayoutGenerationRequest;
use crate::llm::{self};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[utoipa::path(
    post,
    path = "/llm/generate-layout",
    tag = "LLM",
    request_body = LayoutGenerationRequest,
    responses(
        (status = 200, description = "Layout generated successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_token" = []))
)]
pub async fn generate_layout(
    State(_state): State<AppState>,
    Json(req): Json<LayoutGenerationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if req.user_prompt.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "User prompt cannot be empty"
            })),
        ));
    }

    match llm::generate_layout(req).await {
        Ok(response) => Ok((
            StatusCode::OK,
            Json(json!({
                "layout": response.layout
            })),
        )),
        Err(e) => {
            tracing::error!("LLM generation error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to generate layout"
                })),
            ))
        }
    }
}
