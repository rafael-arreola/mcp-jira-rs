use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Content Management (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueContentParams {
    /// Content type: "comment" or "worklog"
    pub content_type: String,

    /// Operation: "add", "update", "delete", "get"
    pub operation: String,

    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Content ID (required for update/delete of specific item, omit for get all or add)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,

    /// Operation payload (structure varies by operation and content type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

// ============================================================================
// PUBLIC API - Social Interactions (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueSocialParams {
    /// Action: "watch", "unwatch", "vote"
    pub action: String,

    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Account ID (required for watch/unwatch, omit for vote)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Comment Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommentData {
    pub body: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<EntityProperty>>,
}

// ============================================================================
// INTERNAL DTOs - Worklog Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorklogData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<EntityProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_users: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_estimate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_estimate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_editable_flag: Option<bool>,
}

// ============================================================================
// SHARED TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Visibility {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntityProperty {
    pub key: String,
    pub value: JsonValue,
}

// ============================================================================
// RESPONSE TYPES - Comments
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub author: Option<User>,
    pub body: Option<JsonValue>,
    pub update_author: Option<User>,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<EntityProperty>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub comments: Vec<Comment>,
}

// ============================================================================
// RESPONSE TYPES - Worklogs
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Worklog {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub issue_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<JsonValue>,
    pub created: String,
    pub updated: String,
    pub started: String,
    pub time_spent: String,
    pub time_spent_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<EntityProperty>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorklogsPage {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub worklogs: Vec<Worklog>,
}

// ============================================================================
// SHARED USER TYPE
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_urls: Option<JsonValue>,
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
}
