use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details about a user.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct User {
    /// The account ID of the user.
    pub account_id: String,
    /// The account type of the user.
    pub account_type: Option<String>,
    /// The display name of the user.
    pub display_name: Option<String>,
    /// The email address of the user.
    pub email_address: Option<String>,
    /// Whether the user is active.
    pub active: Option<bool>,
    /// The time zone of the user.
    pub time_zone: Option<String>,
    /// The locale of the user.
    pub locale: Option<String>,
    /// The groups that the user belongs to.
    pub groups: Option<UserGroups>,
    /// The application roles the user is assigned to.
    pub application_roles: Option<UserApplicationRoles>,
    /// The avatars of the user.
    pub avatar_urls: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct UserGroups {
    pub size: i32,
    pub items: Vec<UserGroup>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct UserGroup {
    pub name: String,
    pub self_link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct UserApplicationRoles {
    pub size: i32,
    pub items: Vec<UserApplicationRole>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct UserApplicationRole {
    pub key: String,
    pub name: String,
}

// --- Search Users ---

/// Parameters for searching users.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchUsersParams {
    /// The search query.
    pub query: String,
    /// The index of the first item to return in a page of results (0-based).
    pub start_at: Option<i32>,
    /// The maximum number of items to return per page.
    pub max_results: Option<i32>,
    /// Whether to include active users.
    pub include_active: Option<bool>,
    /// Whether to include inactive users.
    pub include_inactive: Option<bool>,
}

// --- Get Myself ---

/// Parameters for getting the current user.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetMyselfParams {
    /// Use "groups" to include the groups the user belongs to.
    /// Use "applicationRoles" to include the application roles the user is assigned to.
    pub expand: Option<String>,
}
