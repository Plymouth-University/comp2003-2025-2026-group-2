use anyhow::Result;

const PROFILE_PICTURES_COLLECTION: &str = "profile_pictures";
const COMPANY_LOGOS_COLLECTION: &str = "company_logos";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ProfilePictureDoc {
    _id: mongodb::bson::Uuid,
    user_id: String,
    data: Vec<u8>,
    content_type: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CompanyLogoDoc {
    _id: mongodb::bson::Uuid,
    company_id: String,
    data: Vec<u8>,
    content_type: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upload_profile_picture(
    client: &mongodb::Client,
    data: Vec<u8>,
    user_id: &str,
    content_type: &str,
) -> Result<String> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<ProfilePictureDoc> =
        db.collection(PROFILE_PICTURES_COLLECTION);

    let file_id = mongodb::bson::Uuid::new();

    let doc = ProfilePictureDoc {
        _id: file_id,
        user_id: user_id.to_string(),
        data,
        content_type: content_type.to_string(),
        created_at: chrono::Utc::now(),
    };

    collection.insert_one(doc).await?;

    Ok(file_id.to_string())
}

pub async fn get_profile_picture(
    client: &mongodb::Client,
    file_id: &str,
) -> Result<Option<(String, Vec<u8>)>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<ProfilePictureDoc> =
        db.collection(PROFILE_PICTURES_COLLECTION);

    let oid = mongodb::bson::Uuid::parse_str(file_id)
        .map_err(|e| anyhow::anyhow!("Invalid file ID: {e}"))?;

    let filter = mongodb::bson::doc! { "_id": oid };

    let doc = collection.find_one(filter).await?;

    match doc {
        Some(picture) => Ok(Some((picture.content_type, picture.data))),
        None => Ok(None),
    }
}

pub async fn delete_profile_picture(client: &mongodb::Client, file_id: &str) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<ProfilePictureDoc> =
        db.collection(PROFILE_PICTURES_COLLECTION);

    let oid = mongodb::bson::Uuid::parse_str(file_id)
        .map_err(|e| anyhow::anyhow!("Invalid file ID: {e}"))?;

    let filter = mongodb::bson::doc! { "_id": oid };
    collection.delete_one(filter).await?;

    Ok(())
}

pub async fn upload_company_logo(
    client: &mongodb::Client,
    data: Vec<u8>,
    company_id: &str,
    content_type: &str,
) -> Result<String> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<CompanyLogoDoc> = db.collection(COMPANY_LOGOS_COLLECTION);

    let file_id = mongodb::bson::Uuid::new();

    let doc = CompanyLogoDoc {
        _id: file_id,
        company_id: company_id.to_string(),
        data,
        content_type: content_type.to_string(),
        created_at: chrono::Utc::now(),
    };

    collection.insert_one(doc).await?;

    Ok(file_id.to_string())
}

pub async fn get_company_logo(
    client: &mongodb::Client,
    file_id: &str,
) -> Result<Option<(String, Vec<u8>)>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<CompanyLogoDoc> = db.collection(COMPANY_LOGOS_COLLECTION);

    let oid = mongodb::bson::Uuid::parse_str(file_id)
        .map_err(|e| anyhow::anyhow!("Invalid file ID: {e}"))?;

    let filter = mongodb::bson::doc! { "_id": oid };

    let doc = collection.find_one(filter).await?;

    match doc {
        Some(logo) => Ok(Some((logo.content_type, logo.data))),
        None => Ok(None),
    }
}

pub async fn delete_company_logo(client: &mongodb::Client, file_id: &str) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<CompanyLogoDoc> = db.collection(COMPANY_LOGOS_COLLECTION);

    let oid = mongodb::bson::Uuid::parse_str(file_id)
        .map_err(|e| anyhow::anyhow!("Invalid file ID: {e}"))?;

    let filter = mongodb::bson::doc! { "_id": oid };
    collection.delete_one(filter).await?;

    Ok(())
}
