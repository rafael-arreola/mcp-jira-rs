pub mod agile;
pub mod enums;
pub mod helpers;
pub mod issue;
pub mod jql;
pub mod sprint;
pub mod user;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A safe wrapper for JSON values that produces a compatible schema.
#[derive(Debug, Serialize, Clone, JsonSchema)]
#[serde(transparent)]
#[schemars(inline)]
pub struct JsonValue(pub serde_json::Value);

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