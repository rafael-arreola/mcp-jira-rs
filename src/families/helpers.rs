use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Text to ADF Conversion (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextToAdfParams {
    /// Plain text to convert to Atlassian Document Format (ADF)
    pub text: String,

    /// Text style: "paragraph" (default), "heading1", "heading2", "heading3", "codeblock"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
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

pub fn text_to_adf(text: &str, style: Option<&str>) -> JsonValue {
    match style {
        Some("heading1") => JsonValue(serde_json::json!({
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
        Some("heading2") => JsonValue(serde_json::json!({
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
        Some("heading3") => JsonValue(serde_json::json!({
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
        Some("codeblock") => JsonValue(serde_json::json!({
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
        _ => JsonValue(serde_json::json!({
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
