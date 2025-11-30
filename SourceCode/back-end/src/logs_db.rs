use anyhow::Result;
use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub position: Position,
    pub label: Option<String>,
    pub placeholder: Option<String>,
}

pub type TemplateLayout = Vec<TemplateField>;

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct TemplateDocument {
    pub template_name: String,
    pub template_layout: TemplateLayout,
    pub company_id: String,
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
