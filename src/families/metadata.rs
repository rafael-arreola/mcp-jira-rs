use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PUBLIC API - Metadata Catalog (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MetadataCatalogParams {
    /// Catalog type: "labels", "priorities", "resolutions", "statuses", "issueTypes"
    pub catalog_type: String,

    /// Project key (optional, for project-specific catalogs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Maximum results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

// ============================================================================
// PUBLIC API - Field Discovery (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldDiscoverParams {
    /// Scope: "global", "project", "issueType"
    pub scope: String,

    /// Scope identifier (project key for "project", issue type name for "issueType")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<String>,

    /// Field type filter: "custom", "system", or omit for all
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,

    /// Search term to filter field names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,

    /// Include field options/values (for select fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_options: Option<bool>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Maximum results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

// ============================================================================
// RESPONSE TYPES - Labels
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub values: Vec<Label>,
}

// ============================================================================
// RESPONSE TYPES - Priorities
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_color: Option<String>,
}

// ============================================================================
// RESPONSE TYPES - Resolutions
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ============================================================================
// RESPONSE TYPES - Statuses
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_category: Option<StatusCategory>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusCategory {
    pub id: i64,
    pub key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_name: Option<String>,
}

// ============================================================================
// RESPONSE TYPES - Issue Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueType {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_id: Option<i64>,
}

// ============================================================================
// RESPONSE TYPES - Fields
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clause_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<FieldSchema>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub values: Vec<Field>,
}

// ============================================================================
// RESPONSE TYPES - Field Options
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldOption {
    pub id: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonValue>>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldOptionsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub values: Vec<FieldOption>,
}

// ============================================================================
// RESPONSE TYPES - Create Meta (Field Requirements)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMeta {
    pub projects: Vec<ProjectMeta>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuetypes: Option<Vec<IssueTypeMeta>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueTypeMeta {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, FieldMeta>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldMeta {
    pub required: bool,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<FieldSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_default_value: Option<bool>,
}
