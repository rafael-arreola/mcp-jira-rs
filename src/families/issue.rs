use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PUBLIC API - Specialized Issue Mutations
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(example = "issue_create_example_func()")]
pub struct IssueCreateParams {
    /// Project key (e.g., "PROJ").
    pub project_key: String,
    
    /// Issue type name. Common values: "Task", "Bug", "Story", "Epic".
    pub issue_type: String,

    /// Issue fields. Standard fields are strictly typed. 
    /// Custom fields (e.g., "customfield_10001") are supported via dynamic mapping.
    pub fields: IssueCreateFields,

    /// Optional update operations (rarely used during creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateFields {
    /// The summary (title) of the issue. Required for most issue types.
    pub summary: String,

    /// The description. Can be a String (plain text) or ADF Object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<JsonValue>,

    /// Priority of the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<IdOrName>,

    /// Assignee of the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<AccountId>,

    /// Parent issue (for Sub-tasks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<KeyObject>,

    /// Component(s) associated with the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<IdOrName>>,

    /// Labels for the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    /// Captures any other fields (like customfield_10001) not explicitly defined above.
    #[serde(flatten)]
    pub custom: HashMap<String, JsonValue>,
}

fn issue_create_example_func() -> IssueCreateParams {
    let mut custom = HashMap::new();
    custom.insert("customfield_10001".to_string(), JsonValue(serde_json::json!("Value")));

    IssueCreateParams {
        project_key: "PROJ".to_string(),
        issue_type: "Task".to_string(),
        fields: IssueCreateFields {
            summary: "Fix validation error".to_string(),
            description: Some(JsonValue(serde_json::json!("Description text"))),
            priority: Some(IdOrName { id: None, name: Some("High".to_string()) }),
            assignee: None,
            parent: None,
            components: None,
            labels: Some(vec!["bug".to_string(), "frontend".to_string()]),
            custom,
        },
        update: None,
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(example = "issue_update_example_func()")]
pub struct IssueUpdateParams {
    /// Issue ID or key (e.g., "PROJ-123")
    pub issue_id_or_key: String,

    /// Fields to update. Only provide fields you wish to change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<IssueUpdateFields>,

    /// Advanced update operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JsonValue>,

    /// Whether to send email notifications (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_users: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<JsonValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<IdOrName>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<AccountId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<IdOrName>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    #[serde(flatten)]
    pub custom: HashMap<String, JsonValue>,
}

fn issue_update_example_func() -> IssueUpdateParams {
    IssueUpdateParams {
        issue_id_or_key: "PROJ-123".to_string(),
        fields: Some(IssueUpdateFields {
            summary: Some("New Title".to_string()),
            description: None,
            priority: Some(IdOrName { name: Some("Low".to_string()), id: None }),
            assignee: None,
            components: None,
            labels: None,
            custom: HashMap::new(),
        }),
        update: None,
        notify_users: Some(false),
    }
}

// ============================================================================
// HELPER STRUCTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IdOrName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountId {
    pub account_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KeyObject {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(example = "issue_transition_example_func()")]
pub struct IssueTransitionParams {
    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Transition ID.
    pub transition_id: String,

    /// Fields to set specifically during this transition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<IssueUpdateFields>,
}

fn issue_transition_example_func() -> IssueTransitionParams {
    IssueTransitionParams {
        issue_id_or_key: "PROJ-123".to_string(),
        transition_id: "31".to_string(),
        fields: Some(IssueUpdateFields {
            summary: None,
            description: None,
            priority: None,
            assignee: None,
            components: None,
            labels: None,
            custom: HashMap::new(), // Resolution usually goes here if complex, or strictly typed if we added it
        }),
    }
}


#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssignParams {
    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Account ID of the user to assign (use user_search to find)
    pub account_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteParams {
    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Whether to delete subtasks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_subtasks: Option<bool>,
}

// ============================================================================
// PUBLIC API - New Helper Tools
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueRequiredFieldsParams {
    /// Project key
    pub project_key: String,

    /// Issue type name
    pub issue_type_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequiredFieldInfo {
    pub id: String,
    pub name: String,
    pub required: bool,
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<JsonValue>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldSearchByNameParams {
    /// Visible name of the field (e.g., "Story Points")
    pub query: String,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================


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
