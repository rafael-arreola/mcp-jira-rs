use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgileRankIssuesArgs {
    /// List of issue keys to rank
    pub issue_keys: Vec<String>,
    
    /// Rank after this issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_issue_key: Option<String>,
    
    /// Rank before this issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_issue_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardGetBacklogArgs {
    /// Board name to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_name: Option<String>,
    
    /// Project key to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,
}