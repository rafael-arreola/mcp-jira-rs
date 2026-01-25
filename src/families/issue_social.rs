use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Parameters ---

/// Parameters for adding a watcher.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddWatcherParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The account ID of the user to add as a watcher.
    pub account_id: String,
}

/// Parameters for deleting a watcher.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWatcherParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The account ID of the user to remove as a watcher.
    pub account_id: Option<String>,
}

/// Parameters for basic issue-related social actions.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueSocialParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
}
