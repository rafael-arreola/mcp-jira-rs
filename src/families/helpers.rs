use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Text to ADF Conversion (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum AdfStyle {
    #[default]
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Codeblock,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextToAdfParams {
    /// Plain text to convert to Atlassian Document Format (ADF)
    pub text: String,

    /// Text style
    #[serde(default)]
    pub style: AdfStyle,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdfDocument {
    /// ADF document in JSON format
    pub adf: JsonValue,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub fn text_to_adf(text: &str, style: AdfStyle) -> JsonValue {
    match style {
        AdfStyle::Heading1 => JsonValue(serde_json::json!({
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": { "level": 1 },
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ]
                }
            ]
        })),
        AdfStyle::Heading2 => JsonValue(serde_json::json!({
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": { "level": 2 },
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ]
                }
            ]
        })),
        AdfStyle::Heading3 => JsonValue(serde_json::json!({
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": { "level": 3 },
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ]
                }
            ]
        })),
        AdfStyle::Codeblock => JsonValue(serde_json::json!({
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "codeBlock",
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ]
                }
            ]
        })),
        AdfStyle::Paragraph => JsonValue(serde_json::json!({
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ]
                }
            ]
        })),
    }
}
