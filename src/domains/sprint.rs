use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::enums::SprintState;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardGetSprintsArgs {
    /// Board name to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_name: Option<String>,
    
    /// Project key to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,
    
    /// State of the sprints (active, future, closed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<SprintState>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SprintCreateArgs {
    /// Board ID where the sprint will be created
    pub board_id: i64,
    
    /// Name of the sprint
    pub name: String,
    
    /// Goal of the sprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    
    /// Start date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    
    /// End date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SprintUpdateArgs {
    /// Sprint ID
    pub sprint_id: i64,
    
    /// New name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// New goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    
    /// New state (active, closed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<SprintState>,

    /// Start date (ISO 8601), required when starting a sprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,

    /// End date (ISO 8601), required when starting a sprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SprintAddIssuesArgs {
    /// Sprint ID
    pub sprint_id: i64,
    
    /// List of issue keys to add
    pub issue_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SprintDeleteArgs {
    /// Sprint ID to delete
    pub sprint_id: i64,
}