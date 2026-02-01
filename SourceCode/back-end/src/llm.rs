use crate::dto::{LayoutGenerationRequest, LayoutGenerationResponse};
use crate::logs_db::TemplateLayout;
use anyhow::Result;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use ollama_rs::generation::parameters::{FormatType, JsonStructure};
use ollama_rs::models::ModelOptions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct LayoutOutput {
    pub template_layout: TemplateLayout,
}

const MODEL: &str = "qwen3:4b-instruct";

const SYSTEM_PROMPT: &str = r#"You are a UI Layout Engine. Your goal is to generate a JSON layout for a dashboard based on the user's request and the provided context.

### CONTEXT AWARENESS:
The user will provide:
1. **Canvas Dimensions**: The maximum width/height available.
2. **Component Sizes**: The specific Width x Height for each field type (e.g., temperature is 80px tall).

### LAYOUT RULES:
1. **Vertical Flow**: Arrange items in a single vertical column unless asked for a grid.
2. **Positioning Logic**:
   - Start the first item at `{"x": 20, "y": 20}`.
   - **Calculate Y**: For the next item, take the previous item's `y` + previous item's `height` + 20px gap.
   - *Example*: If Item 1 is at y=20 and is 80px tall, Item 2 starts at y=120 (20+80+20).
   - Keep `x: 20` aligned left.

3. **Strict Field Types** (Use ONLY these):
   - `text_input`: Standard text field.
   - `checkbox`: Boolean toggle.
   - `temperature`: Temperature picker (Unit defaults to "°C").
   - `dropdown`: Selection list (Must provide `options` array).
   - `label`: Static text.

4. **Props Handling**:
   - `min`/`max` must be numbers (e.g., `min: -10.0`).
   - `editable`: usually `true` for inputs, `false` for labels.
   - `value`: Default value if specified.

### OUTPUT FORMAT:
Return ONLY the raw JSON object matching the `template_layout` schema. Do not include markdown formatting."#;

/// Generates a layout using the configured LLM.
///
/// # Errors
/// Returns an error if the LLM request fails or if environment variables are missing.
pub async fn generate_layout(request: LayoutGenerationRequest) -> Result<LayoutGenerationResponse> {
    let ollama = Ollama::from_url(Url::parse(
        &std::env::var("OLLAMA_URL").unwrap_or("http://127.0.0.1:11434".to_string()),
    )?);

    let messages = vec![
        ChatMessage::new(MessageRole::System, SYSTEM_PROMPT.to_string()),
        ChatMessage::new(MessageRole::User, request.user_prompt),
    ];

    let format = FormatType::StructuredJson(Box::new(JsonStructure::new::<LayoutOutput>()));

    let options = ModelOptions::default()
        .temperature(0.2)
        .top_p(0.8)
        .top_k(20);

    let req = ChatMessageRequest::new(MODEL.to_string(), messages)
        .format(format)
        .options(options);

    let res = ollama.send_chat_messages(req).await?;

    let layout: serde_json::Value = serde_json::from_str(&res.message.content)
        .unwrap_or_else(|_| serde_json::json!({ "template_layout": [] }));

    Ok(LayoutGenerationResponse { layout })
}
