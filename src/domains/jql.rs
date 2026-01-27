use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchIssuesArgs {
    /// Fuzzy search text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    /// Raw JQL query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jql: Option<String>,
    
    /// Filter by status name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    
    /// Filter by assignee (me, unassigned, or accountId)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    
    /// Maximum results (default 50)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}