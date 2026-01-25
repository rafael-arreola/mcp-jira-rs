use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Projects ---

/// Details of a project.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Project {
    /// The ID of the project.
    pub id: String,
    /// The key of the project.
    pub key: String,
    /// The name of the project.
    pub name: String,
    /// The type of the project.
    pub project_type_key: Option<String>,
    /// The URL of the project details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The URLs of the project's avatars.
    pub avatar_urls: Option<JsonValue>,
    /// Whether the project is simplified.
    pub simplified: Option<bool>,
    /// The style of the project.
    pub style: Option<String>,
    /// Whether the project is private.
    pub is_private: Option<bool>,
}

/// Parameters for searching projects.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchProjectsParams {
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
    pub order_by: Option<String>,
    pub id: Option<Vec<i64>>,
    pub keys: Option<Vec<String>>,
    pub query: Option<String>,
    pub type_key: Option<String>,
    pub category_id: Option<i64>,
    pub action: Option<String>,
    pub expand: Option<String>,
}

/// Paged response for projects.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanProject {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<Project>,
}

// --- Versions ---

/// Details of a project version.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Version {
    /// The URL of the version.
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    /// The ID of the version.
    pub id: Option<String>,
    /// The name of the version.
    pub name: Option<String>,
    /// The description of the version.
    pub description: Option<String>,
    /// Whether the version is archived.
    pub archived: Option<bool>,
    /// Whether the version is released.
    pub released: Option<bool>,
    /// The release date of the version.
    pub release_date: Option<String>,
    /// The start date of the version.
    pub start_date: Option<String>,
    /// The ID of the project the version belongs to.
    pub project_id: Option<i64>,
}

/// Parameters for getting a project's versions.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetProjectVersionsParams {
    /// The ID or key of the project.
    pub project_id_or_key: String,
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
    pub order_by: Option<String>,
    pub query: Option<String>,
    pub status: Option<String>,
    pub expand: Option<String>,
}

/// Paged response for versions.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanVersion {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<Version>,
}

/// Parameters for creating a version.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionParams {
    /// The name of the version.
    pub name: String,
    /// The ID or key of the project.
    pub project: String,
    /// The description of the version.
    pub description: Option<String>,
    /// The release date of the version.
    pub release_date: Option<String>,
    /// The start date of the version.
    pub start_date: Option<String>,
    /// Whether the version is archived.
    pub archived: Option<bool>,
    /// Whether the version is released.
    pub released: Option<bool>,
}

// --- Components ---

/// Details of a project component.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Component {
    /// The URL of the component.
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    /// The ID of the component.
    pub id: Option<String>,
    /// The name of the component.
    pub name: Option<String>,
    /// The description of the component.
    pub description: Option<String>,
    /// The key of the project the component belongs to.
    pub project: Option<String>,
    /// The ID of the project the component belongs to.
    pub project_id: Option<i64>,
}

/// Parameters for getting a project's components.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetProjectComponentsParams {
    /// The ID or key of the project.
    pub project_id_or_key: String,
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
    pub order_by: Option<String>,
    pub query: Option<String>,
}

/// Paged response for components.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanComponentWithIssueCount {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<Component>,
}

/// Parameters for creating a component.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentParams {
    /// The name of the component.
    pub name: String,
    /// The ID or key of the project.
    pub project: String,
    /// The description of the component.
    pub description: Option<String>,
    /// The account ID of the component's lead user.
    pub lead_account_id: Option<String>,
    /// The assigneen type for issues in this component.
    pub assignee_type: Option<String>,
}

// --- Roles ---

/// Details of a project role.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct ProjectRole {
    /// The URL of the role.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The name of the role.
    pub name: String,
    /// The ID of the role.
    pub id: i64,
    /// The description of the role.
    pub description: Option<String>,
}
