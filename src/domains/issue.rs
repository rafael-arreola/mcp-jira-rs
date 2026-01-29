use super::enums;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateArgs {
    /// Project key (e.g., "PROJ").
    pub project_key: String,
    
    /// Issue type. Values: Story, Bug, Epic, Task, Sub-task.
    pub issue_type: enums::IssueType,

    /// The issue title.
    pub summary: String,

    /// Plain text description (will be auto-converted to ADF).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Priority. Values: Highest, High, Medium, Low, Lowest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<enums::Priority>,

    /// Parent key. Required for Sub-tasks. For Stories, it can link to an Epic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,

    /// List of string tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    /// List of component names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<String>>,

    /// Story Points (Classic/Company-managed projects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_points: Option<f64>,

    /// Story point estimate (Next-Gen/Team-managed projects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_point_estimate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateStatusArgs {
    /// Issue ID or key (e.g., "PROJ-123").
    pub issue_key: String,
    
    /// Target status. Values: To Do, In Progress, Done, In Review, Blocked, Cancelled.
    pub status: enums::Status,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssignArgs {
    /// Issue ID or key.
    pub issue_key: String,
    
    /// Assignee identifier: "me", "unassigned", or Account ID.
    pub assignee: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueEditDetailsArgs {
    /// Issue ID or key.
    pub issue_key: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_type: Option<enums::IssueType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<enums::Priority>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueSetStoryPointsArgs {
    /// Issue ID or key (e.g., "PROJ-123").
    pub issue_key: String,

    /// Numeric estimation.
    pub story_points: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueAddCommentArgs {
    /// Issue ID or key.
    pub issue_key: String,
    
    /// Comment text.
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueLinkArgs {
    /// Source issue key.
    pub source_issue_key: String,
    
    /// Target issue key.
    pub target_issue_key: String,
    
    /// Link type. Values: Blocks, Is blocked by, Clones, Relates, Duplicates.
    pub link_type: enums::LinkType,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueLogWorkArgs {
    /// Issue ID or key.
    pub issue_key: String,
    
    /// Time spent. E.g., "1h 30m".
    pub time_spent: String,
    
    /// Started date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<String>,
    
    /// Optional comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteArgs {
    /// Issue ID or key.
    pub issue_key: String,
    
    /// Whether to delete subtasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_subtasks: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchiveArgs {
    /// List of issue keys or IDs to archive.
    pub issue_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueUnarchiveArgs {
    /// List of issue keys or IDs to unarchive.
    pub issue_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteCommentArgs {
    /// Issue ID or key.
    pub issue_key: String,
    
    /// Comment ID.
    pub comment_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteLinkArgs {
    /// Link ID.
    pub link_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueSetParentArgs {
    /// Issue key to link (e.g., "PROJ-123" - the Story).
    pub issue_key: String,

    /// Parent issue key (e.g., "PROJ-100" - the Epic). Set to empty string to remove parent.
    pub parent_key: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueGetArgs {
    /// Issue ID or key (e.g., "PROJ-123").
    pub issue_key: String,

    /// **[OPTIONAL]** Field filter to reduce LLM context usage by ~70-90%.
    ///
    /// **Presets** (recommended for most cases):
    /// - `"minimal"`: Only id, key (~95% reduction)
    /// - `"basic"`: id, key, summary, status (~90% reduction)
    /// - `"standard"`: basic + assignee, priority, dates (~85% reduction)
    /// - `"detailed"`: standard + description, labels, components (~70% reduction)
    /// - `"full"`: No filter (default behavior)
    ///
    /// **Custom field list** (space or comma separated):
    /// - `"id key summary status"` or `"id,key,summary,status"`
    /// - Supports dot notation: `"assignee.displayName"`
    /// - Include custom fields: `"id key customfield_10016"` (use `fields_list` to discover)
    ///
    /// **Examples**:
    /// ```json
    /// {"issue_key": "PROJ-123", "filter": "basic"}
    /// {"issue_key": "PROJ-123", "filter": "id key summary customfield_10016"}
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransitionResponse {
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Transition {
    pub id: String,
    pub name: String,
    pub to: TransitionTo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransitionTo {
    pub name: String,
    pub status_category: Option<StatusCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatusCategory {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FieldsListArgs {
    /// **[OPTIONAL]** Filter by field type.
    /// - `"system"`: Only Jira system fields (summary, status, assignee, etc.)
    /// - `"custom"`: Only custom fields (Story Points, Sprint, etc.)
    /// - Omit for all fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
}
