use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::JsonValue;
use super::issue_comment::{EntityProperty, Visibility};

/// Details of a worklog.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Worklog {
    /// The ID of the worklog.
    pub id: String,
    /// The ID of the issue the worklog belongs to.
    pub issue_id: String,
    /// The user who created the worklog.
    pub author: Option<WorklogUser>,
    /// The user who last updated the worklog.
    pub update_author: Option<WorklogUser>,
    /// The comment for the worklog (ADF format).
    pub comment: Option<JsonValue>,
    /// The datetime the worklog was created.
    pub created: String,
    /// The datetime the worklog was last updated.
    pub updated: String,
    /// The datetime the worklog started.
    pub started: String,
    /// The time spent as a string (e.g. "3h 20m").
    pub time_spent: String,
    /// The time spent in seconds.
    pub time_spent_seconds: i64,
    /// The visibility of the worklog.
    pub visibility: Option<Visibility>,
    /// The properties of the worklog.
    pub properties: Option<Vec<EntityProperty>>,
    /// The URL of the worklog details.
    #[serde(rename = "self")]
    pub self_link: String,
}

/// Details about a user in the context of a worklog.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorklogUser {
    pub account_id: String,
    pub display_name: Option<String>,
    pub active: Option<bool>,
    pub avatar_urls: Option<JsonValue>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Paginated list of worklogs.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanWorklog {
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub worklogs: Vec<Worklog>,
}

// --- Parameters ---

/// Parameters for getting worklogs for an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetWorklogsParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The index of the first item to return (0-based).
    pub start_at: Option<i64>,
    /// The maximum number of items to return.
    pub max_results: Option<i32>,
    /// The date and time (as milliseconds since epoch) after which worklogs must have started.
    pub started_after: Option<i64>,
    /// The date and time (as milliseconds since epoch) before which worklogs must have started.
    pub started_before: Option<i64>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}

/// Parameters for adding a worklog.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddWorklogParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The comment for the worklog (ADF format).
    pub comment: Option<JsonValue>,
    /// The datetime the worklog started (ISO 8601).
    pub started: Option<String>,
    /// The time spent as a string (e.g. "3h 20m").
    pub time_spent: Option<String>,
    /// The time spent in seconds.
    pub time_spent_seconds: Option<i64>,
    /// The visibility of the worklog.
    pub visibility: Option<Visibility>,
    /// The properties of the worklog.
    pub properties: Option<Vec<EntityProperty>>,
    /// Whether to notify users of the update.
    pub notify_users: Option<bool>,
    /// How to adjust the remaining estimate: "auto", "new", "manual", "leave".
    pub adjust_estimate: Option<String>,
    /// The new remaining estimate value (if adjustEstimate is "new").
    pub new_estimate: Option<String>,
    /// The value to reduce the remaining estimate by (if adjustEstimate is "manual").
    pub reduce_by: Option<String>,
    /// Whether to override the workflow editable flag.
    pub override_editable_flag: Option<bool>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}

/// Parameters for updating a worklog.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorklogParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The ID of the worklog.
    pub worklog_id: String,
    /// The comment for the worklog (ADF format).
    pub comment: Option<JsonValue>,
    /// The datetime the worklog started (ISO 8601).
    pub started: Option<String>,
    /// The time spent as a string (e.g. "3h 20m").
    pub time_spent: Option<String>,
    /// The time spent in seconds.
    pub time_spent_seconds: Option<i64>,
    /// The visibility of the worklog.
    pub visibility: Option<Visibility>,
    /// The properties of the worklog.
    pub properties: Option<Vec<EntityProperty>>,
    /// Whether to notify users of the update.
    pub notify_users: Option<bool>,
    /// How to adjust the remaining estimate: "auto", "new", "manual", "leave".
    pub adjust_estimate: Option<String>,
    /// The new remaining estimate value.
    pub new_estimate: Option<String>,
    /// Whether to override the workflow editable flag.
    pub override_editable_flag: Option<bool>,
    /// Expand to include additional information.
    pub expand: Option<String>,
}
