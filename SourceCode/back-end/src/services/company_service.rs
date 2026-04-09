use crate::{db, images_db};
use sqlx::PgPool;

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

#[derive(Debug)]
pub enum CompanyServiceError {
    FileTooLarge,
    NoFileProvided,
    NotAnImage,
    CompanyNotFound,
    Internal(String),
}

impl std::fmt::Display for CompanyServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooLarge => write!(f, "File too large. Maximum size is 10MB"),
            Self::NoFileProvided => write!(f, "No file provided"),
            Self::NotAnImage => write!(f, "File must be an image"),
            Self::CompanyNotFound => write!(f, "Company not found"),
            Self::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

pub struct CompanyService;

impl CompanyService {
    pub async fn upload_company_logo(
        postgres: &PgPool,
        mongodb: &mongodb::Client,
        company_id: &str,
        data: Vec<u8>,
    ) -> Result<String, CompanyServiceError> {
        if data.len() > 10 * 1024 * 1024 {
            return Err(CompanyServiceError::FileTooLarge);
        }

        if data.is_empty() {
            return Err(CompanyServiceError::NoFileProvided);
        }

        let content_type = infer_content_type(&data);
        if !content_type.starts_with("image/") {
            return Err(CompanyServiceError::NotAnImage);
        }

        let company = db::get_company_by_id(postgres, company_id)
            .await
            .map_err(|e| CompanyServiceError::Internal(format!("Database error: {e}")))?
            .ok_or(CompanyServiceError::CompanyNotFound)?;

        let file_id = images_db::upload_company_logo(mongodb, data, company_id, &content_type)
            .await
            .map_err(|e| CompanyServiceError::Internal(format!("Failed to upload logo: {e}")))?;

        if let Err(err) = db::update_company_logo_id(postgres, company_id, Some(&file_id)).await {
            tracing::error!("Failed to update company logo: {:?}", err);
            if let Err(delete_err) = images_db::delete_company_logo(mongodb, &file_id).await {
                tracing::error!("Failed to cleanup uploaded logo: {:?}", delete_err);
            }
            return Err(CompanyServiceError::Internal("Failed to update logo reference".to_string()));
        }

        if let Some(old_logo_id) = &company.logo_id
            && let Err(err) = images_db::delete_company_logo(mongodb, old_logo_id).await
        {
            tracing::error!("Failed to delete old company logo: {:?}", err);
        }

        Ok(file_id)
    }
}
