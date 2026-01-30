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

use super::enums::FieldPreset;

/// Parsea un string de filtro en formato Jira API.
/// Soporta:
/// - Presets: "basic", "standard", "minimal"
/// - Listas: "id key summary" o "id,key,summary"
/// - Sin cambios: "*all" o cualquier otro formato Jira
pub fn parse_field_filter(filter: &str) -> String {
    let trimmed = filter.trim();

    // Intentar parsear como preset
    match trimmed.to_lowercase().as_str() {
        "minimal" => return FieldPreset::Minimal.to_field_list(),
        "basic" => return FieldPreset::Basic.to_field_list(),
        "standard" => return FieldPreset::Standard.to_field_list(),
        "detailed" => return FieldPreset::Detailed.to_field_list(),
        "full" => return FieldPreset::Full.to_field_list(),
        _ => {}
    }

    // Si contiene espacios pero no comas, convertir a formato Jira
    if trimmed.contains(' ') && !trimmed.contains(',') {
        return trimmed.replace(' ', ",");
    }

    // Retornar sin cambios (ya está en formato Jira)
    trimmed.to_string()
}
