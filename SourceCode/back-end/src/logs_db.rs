use anyhow::Result;
use mongodb::bson::Uuid;
use utoipa::ToSchema;
use futures_util::TryStreamExt;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateFieldProps {
    pub text: Option<String>,
    pub size: Option<u32>,
    pub weight: Option<String>,
    pub value: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
    pub selected: Option<bool>,
    pub options: Option<Vec<String>>,
    pub editable: Option<bool>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateField {
    #[serde(rename = "type")]
    pub field_type: String,
    pub position: Position,
    pub props: TemplateFieldProps,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct Schedule {
    pub frequency: Frequency,
    pub days_of_week: Option<Vec<u8>>, // for daily schedule (one per day)
    pub day_of_week: Option<u8>,        // for weekly schedule (one per week)
    pub day_of_month: Option<u8>,   // for monthly schedule (one per month)
    pub month_of_year: Option<u8>, // for yearly schedule (one per year)
}

pub type TemplateLayout = Vec<TemplateField>;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateDocument {
    pub template_name: String,
    pub template_layout: TemplateLayout,
    pub company_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub schedule: Schedule,
    pub created_by: Uuid,
}

pub async fn init_mongodb() -> Result<mongodb::Client> {
    let mongo_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set in environment");
    mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .map_err(Into::into)
}

pub async fn add_template(client: &mongodb::Client, template: &TemplateDocument) -> Result<()> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    collection.insert_one(template).await?;
    Ok(())
}

pub async fn get_template_by_name(
    client: &mongodb::Client,
    template_name: &str,
    company_id: &str,
) -> Result<Option<TemplateDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "template_name": template_name,
        "company_id": company_id,
    };

    let result = collection.find_one(filter).await?;
    Ok(result)
}

pub async fn get_templates_by_company(
    client: &mongodb::Client,
    company_id: &str,
) -> Result<Vec<TemplateDocument>> {
    let db = client.database("logs_db");
    let collection: mongodb::Collection<TemplateDocument> = db.collection("templates");

    let filter = mongodb::bson::doc! {
        "company_id": company_id,
    };

    let mut cursor = collection.find(filter).await?;
    let mut templates = Vec::new();

    while let Some(template) = cursor.try_next().await? {
        templates.push(template);
    }

    Ok(templates)
}
