pub mod agile;
pub mod issue;
pub mod issue_attachment;
pub mod issue_comment;
pub mod issue_custom_field;
pub mod issue_field;
pub mod issue_link;
pub mod issue_metadata;
pub mod issue_social;
pub mod issue_worklog;
pub mod jql;
pub mod project;
pub mod user;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A safe wrapper for JSON values that produces a compatible schema (String).
#[derive(Debug, Serialize, Clone)]
#[serde(transparent)]
pub struct JsonValue(pub serde_json::Value);

impl JsonSchema for JsonValue {
    fn schema_name() -> Cow<'static, str> {
        "JsonValue".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

impl<'de> Deserialize<'de> for JsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::String(s) => match serde_json::from_str(&s) {
                Ok(parsed) => Ok(JsonValue(parsed)),
                Err(_) => Ok(JsonValue(serde_json::Value::String(s))),
            },
            _ => Ok(JsonValue(v)),
        }
    }
}

impl std::ops::Deref for JsonValue {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
