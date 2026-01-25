use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details of a field.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Field {
    /// The ID of the field.
    pub id: String,
    /// The name of the field.
    pub name: String,
    /// The description of the field.
    pub description: Option<String>,
    /// Whether the field is a custom field.
    pub custom: Option<bool>,
    /// Whether the field is searchable.
    pub searchable: Option<bool>,
    /// Whether the field is sortable.
    pub sortable: Option<bool>,
    /// The names that can be used to reference the field in JQL.
    pub clause_names: Option<Vec<String>>,
    /// The schema of the field.
    pub schema: Option<FieldSchema>,
}

/// The schema of a field.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct FieldSchema {
    /// The data type of the field.
    #[serde(rename = "type")]
    pub type_name: String,
    /// If the field is a custom field, the URI of the custom field type.
    pub custom: Option<String>,
    /// If the field is a custom field, the custom field ID.
    pub custom_id: Option<i64>,
    /// If the field is a list, the data type of the items in the list.
    pub items: Option<String>,
    /// If the field is a system field, the name of the system field.
    pub system: Option<String>,
}
