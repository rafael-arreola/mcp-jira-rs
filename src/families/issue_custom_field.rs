use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Custom Field Options ---

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct CustomFieldOption {
    pub id: String,
    pub value: String,
    pub disabled: bool,
    pub option_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanCustomFieldOption {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<CustomFieldOption>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetCustomFieldOptionsParams {
    pub field_id: String,
    pub context_id: i64,
    pub option_id: Option<i64>,
    pub only_options: Option<bool>,
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
}
