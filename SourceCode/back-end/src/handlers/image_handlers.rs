use crate::{
    AppState,
    dto::{
        ErrorResponse,
        ImageUploadResponse,
    },
    logs_db,
    middleware::AuthToken,
    db,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    extract::Multipart,
    response::Response,
    body::Body,
};
use mongodb::bson::oid::ObjectId;
use tracing::error;

pub async fn upload_image(
    AuthToken(claims): AuthToken,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageUploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = db::get_user_by_id(&state.postgres, &claims.user_id)
        .await
        .map_err(|e| {
            error!("Database error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "User not found".to_string(),
            }),
        ))?;

    let company_id = user.company_id.clone().ok_or((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: "User is not associated with a company".to_string(),
        }),
    ))?;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Error parsing multipart: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid multipart data".to_string(),
            }),
        )
    })? {
        let name = field.name().unwrap_or("");
        
        if name == "image" {
            let filename = field.file_name().unwrap_or("image").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            
            let data = field.bytes().await.map_err(|e| {
                error!("Error reading file: {:?}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Error reading file".to_string(),
                    }),
                )
            })?;

            let file_size = data.len() as u64;

            if file_size > 5 * 1024 * 1024 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "File size exceeds 5MB limit".to_string(),
                    }),
                ));
            }

            // Store the image in MongoDB GridFS
            let object_id = logs_db::store_image(
                &state.mongodb,
                data.to_vec(),
                &filename,
                &content_type,
                &claims.user_id,
                &company_id,
            )
            .await
            .map_err(|e| {
                error!("Error storing image: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Error storing image".to_string(),
                    }),
                )
            })?;

            return Ok(Json(ImageUploadResponse {
                filename,
                object_id: object_id.to_string(),
                file_size,
            }));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No image file found in request".to_string(),
        }),
    ))
}

pub async fn get_image(
    AuthToken(_claims): AuthToken,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response<Body>, (StatusCode, Json<ErrorResponse>)> {
    let object_id = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid image ID format".to_string(),
            }),
        )
    })?;

    match logs_db::get_image(&state.mongodb, &object_id).await {
        Ok(Some((data, file))) => {
            let content_type = file.metadata
                .as_ref()
                .and_then(|m| m.get_str("contentType").ok())
                .unwrap_or("application/octet-stream");
            
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Content-Length", data.len().to_string())
                .body(Body::from(data))
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Error building response".to_string(),
                        }),
                    )
                })
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Image not found".to_string(),
            }),
        )),
        Err(e) => {
            error!("Error retrieving image: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Error retrieving image".to_string(),
                }),
            ))
        }
    }
}

pub async fn delete_image(
    AuthToken(_claims): AuthToken,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let object_id = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid image ID format".to_string(),
            }),
        )
    })?;

    logs_db::delete_image(&state.mongodb, &object_id)
        .await
        .map_err(|e| {
            error!("Error deleting image: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Error deleting image".to_string(),
                }),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use mongodb::bson::oid::ObjectId;

    #[test]
    fn test_object_id_parsing_valid() {
        let valid_id = "507f1f77bcf86cd799439011";
        let result = ObjectId::parse_str(valid_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_id_parsing_invalid() {
        let invalid_id = "not-a-valid-object-id";
        let result = ObjectId::parse_str(invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_object_id_parsing_empty() {
        let empty_id = "";
        let result = ObjectId::parse_str(empty_id);
        assert!(result.is_err());
    }
}
