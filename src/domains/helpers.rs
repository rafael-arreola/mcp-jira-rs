use super::JsonValue;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum AdfStyle {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Codeblock,
}

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