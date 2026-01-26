use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Project Queries (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectQueryParams {
    /// Resource type: "project", "versions", "components", "roles", "issueTypes"
    pub resource: String,

    /// Project key or ID (required for versions/components/roles/issueTypes, optional for project)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Maximum results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// Expand options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
}

// ============================================================================
// PUBLIC API - Project Management (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManageParams {
    /// Resource type: "version" or "component"
    pub resource: String,

    /// Operation: "create", "update", "delete"
    pub operation: String,

    /// Project key
    pub project_key: String,

    /// Resource ID (required for update/delete, omit for create)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// Operation payload (structure varies by resource and operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

// ============================================================================
// INTERNAL DTOs - Version Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VersionData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
}

// ============================================================================
// INTERNAL DTOs - Component Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<ComponentLead>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<ComponentLead>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentLead {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

// ============================================================================
// RESPONSE TYPES - Projects
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<ProjectUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_urls: Option<AvatarUrls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUser {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvatarUrls {
    #[serde(rename = "48x48")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_48: Option<String>,
    #[serde(rename = "24x24")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_24: Option<String>,
    #[serde(rename = "16x16")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_16: Option<String>,
    #[serde(rename = "32x32")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_32: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub values: Vec<Project>,
}

// ============================================================================
// RESPONSE TYPES - Versions
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overdue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
}

// ============================================================================
// RESPONSE TYPES - Components
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead: Option<ProjectUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<ProjectUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
}

// ============================================================================
// RESPONSE TYPES - Roles
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRole {
    pub id: i64,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actors: Option<Vec<RoleActor>>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleActor {
    pub id: i64,
    pub display_name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_user: Option<ProjectUser>,
}
