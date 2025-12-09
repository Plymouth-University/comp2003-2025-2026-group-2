use crate::dto::{LayoutGenerationRequest, LayoutGenerationResponse};
use crate::logs_db::TemplateLayout;
use anyhow::Result;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::parameters::{FormatType, JsonStructure};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct LayoutOutput {
    pub template_layout: TemplateLayout,
}

const MODEL: &str = "gemma3:4b";

const SYSTEM_PROMPT: &str = r#"IMPORTANT: You MUST respond with ONLY valid JSON. No text, no explanations, no markdown.

You are generating a template layout in JSON format with REAL VALUES, not placeholders.

Field type requirements:
- text_input: text, size, weight, value, min, max, unit, editable (others: null)
- checkbox: text, size, weight, selected, editable (others: null)
- temperature: value, min, max, unit (all others: null)
- dropdown: text, size, weight, selected, options, editable (others: null)
- label: text, size, weight, editable (others: null)

EXAMPLE with REAL VALUES (not a template):
{
  "template_layout": [
    {
      "field_type": "text_input",
      "position": { "x": 10, "y": 10 },
      "props": {
        "text": "Name",
        "size": "16px",
        "weight": "bold",
        "value": "",
        "min": null,
        "max": null,
        "unit": null,
        "selected": null,
        "options": null,
        "editable": true
      }
    },
    {
      "field_type": "temperature",
      "position": { "x": 10, "y": 50 },
      "props": {
        "text": null,
        "size": null,
        "weight": null,
        "value": "20",
        "min": -50,
        "max": 50,
        "unit": "°C",
        "selected": null,
        "options": null,
        "editable": null
      }
    }
  ]
}

Generate a template layout based on the user's request using REAL VALUES. Return ONLY the JSON object."#;

pub async fn generate_layout(request: LayoutGenerationRequest) -> Result<LayoutGenerationResponse> {
    let ollama = Ollama::from_url(
        Url::parse(&std::env::var("OLLAMA_URL").unwrap_or("http://127.0.0.1:11434".to_string()))
            .unwrap(),
    );
    let prompt = format!("{}\n\nUser request: {}", SYSTEM_PROMPT, request.user_prompt);

    let format = FormatType::StructuredJson(Box::new(JsonStructure::new::<LayoutOutput>()));

    let res = ollama
        .generate(GenerationRequest::new(MODEL.to_string(), prompt).format(format))
        .await?;

    let layout: serde_json::Value = serde_json::from_str(&res.response).unwrap_or_else(|_| {
        serde_json::json!({
            "template_layout": []
        })
    });

    Ok(LayoutGenerationResponse { layout })
}
