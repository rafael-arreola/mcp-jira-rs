use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Restriction for groups or roles.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Visibility {
    /// The type of visibility restriction.
    pub r#type: String,
    /// The value of the visibility restriction.
    pub value: String,
    /// The identifier of the group or role.
    pub identifier: Option<String>,
}

/// Details about a comment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Comment {
    /// The ID of the comment.
    pub id: String,
    /// The URL of the comment details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The user who created the comment.
    pub author: Option<CommentUser>,
    /// The body of the comment (ADF format).
    pub body: JsonValue,
    /// The user who last updated the comment.
    pub update_author: Option<CommentUser>,
    /// The datetime the comment was created.
    pub created: String,
    /// The datetime the comment was last updated.
    pub updated: Option<String>,
    /// The visibility of the comment.
    pub visibility: Option<Visibility>,
    /// The properties of the comment.
    pub properties: Option<Vec<EntityProperty>>,
}

/// Details about a user in the context of a comment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct CommentUser {
    pub account_id: String,
    pub display_name: Option<String>,
    pub active: Option<bool>,
    pub avatar_urls: Option<JsonValue>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Generic entity property.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct EntityProperty {
    pub key: String,
    pub value: JsonValue,
}

/// Paginated list of comments (PageBean format).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanComment {
    pub is_last: Option<bool>,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    #[serde(alias = "comments")]
    pub values: Vec<Comment>,
}

// --- Parameters ---

/// Parameters for adding a comment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The body of the comment (ADF format).
    pub body: JsonValue,
    /// The visibility of the comment.
    pub visibility: Option<Visibility>,
    /// The properties of the comment.
    pub properties: Option<Vec<EntityProperty>>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}

/// Parameters for updating a comment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The ID of the comment.
    pub comment_id: String,
    /// The body of the comment (ADF format).
    pub body: JsonValue,
    /// The visibility of the comment.
    pub visibility: Option<Visibility>,
    /// The properties of the comment.
    pub properties: Option<Vec<EntityProperty>>,
    /// Whether to notify users of the update.
    pub notify_users: Option<bool>,
    /// Whether to override the workflow editable flag.
    pub override_editable_flag: Option<bool>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}

/// Parameters for deleting a comment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCommentParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The ID of the comment.
    pub comment_id: String,
}

/// Parameters for getting comments.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetCommentsParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The index of the first item to return (0-based).
    pub start_at: Option<i64>,
    /// The maximum number of items to return.
    pub max_results: Option<i32>,
    /// The field used to order the results.
    pub order_by: Option<String>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}
