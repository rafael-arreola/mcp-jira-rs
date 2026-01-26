use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PUBLIC API - Issue Mutations (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub enum IssueOperation {
    Create,
    Update,
    Delete,
    Assign,
    Transition,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueMutateParams {
    /// Operation type: "create", "update", "delete", "assign", "transition"
    pub operation: IssueOperation,

    /// Issue ID or key (required for update/delete/assign/transition, omit for create)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id_or_key: Option<String>,

    /// Operation payload (structure varies by operation)
    pub data: JsonValue,

    /// Enable bulk mode (provide array of data objects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulk: Option<bool>,
}

// ============================================================================
// PUBLIC API - Issue Queries (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueQueryParams {
    /// Issue ID or key to fetch (omit to search with JQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id_or_key: Option<String>,

    /// JQL query string (used when issueIdOrKey is omitted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jql: Option<String>,

    /// Fields to return for each issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    /// Expand options (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,

    /// Include available transitions for the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_transitions: Option<bool>,

    /// Maximum results for search (default 50, max 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Properties to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,
}

// ============================================================================
// PUBLIC API - Issue Metadata Discovery
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadataParams {
    /// Project key or ID
    pub project_key: String,

    /// Issue type name to get specific metadata (omit for all types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_type_name: Option<String>,

    /// Expand options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Create Operation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateIssueData {
    pub fields: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JsonValue>,
}

// ============================================================================
// INTERNAL DTOs - Update Operation
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateIssueData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_users: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_editable_flag: Option<bool>,
}

// ============================================================================
// INTERNAL DTOs - Delete Operation
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteIssueData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_subtasks: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Assign Operation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssignIssueData {
    pub account_id: String,
}

// ============================================================================
// INTERNAL DTOs - Transition Operation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransitionIssueData {
    pub transition: TransitionRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransitionRef {
    pub id: String,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_fields: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkCreatedIssues {
    pub issues: Vec<CreatedIssue>,
    pub errors: Vec<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<TransitionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, JsonValue>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_category: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionsResponse {
    pub transitions: Vec<Transition>,
}
