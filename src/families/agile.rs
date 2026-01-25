use crate::families::issue::Issue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Boards ---

/// Details of a board.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    /// The ID of the board.
    pub id: i64,
    /// The URL of the board details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The name of the board.
    pub name: String,
    /// The type of the board (e.g. "scrum", "kanban").
    pub r#type: String,
}

/// Paged response for boards.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanBoard {
    pub max_results: i32,
    pub start_at: i64,
    pub total: Option<i64>,
    pub is_last: bool,
    pub values: Vec<Board>,
}

/// Parameters for getting boards.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBoardsParams {
    /// The starting index of the returned boards.
    pub start_at: Option<i64>,
    /// The maximum number of boards to return per page.
    pub max_results: Option<i32>,
    /// Filters results to boards of the specified type.
    pub r#type: Option<String>,
    /// Filters results to boards that match the specified name.
    pub name: Option<String>,
    /// Filters results to boards that are relevant to a project.
    pub project_key_or_id: Option<String>,
}

/// Parameters for getting a specific board.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBoardParams {
    /// The ID of the board.
    pub board_id: i64,
}

// --- Sprints ---

/// Details of a sprint.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    /// The ID of the sprint.
    pub id: i64,
    /// The URL of the sprint details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The state of the sprint.
    pub state: String,
    /// The name of the sprint.
    pub name: String,
    /// The start date of the sprint.
    pub start_date: Option<String>,
    /// The end date of the sprint.
    pub end_date: Option<String>,
    /// The completion date of the sprint.
    pub complete_date: Option<String>,
    /// The ID of the board the sprint belongs to.
    pub origin_board_id: Option<i64>,
    /// The goal of the sprint.
    pub goal: Option<String>,
}

/// Parameters for getting all sprints for a board.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBoardSprintsParams {
    /// The ID of the board.
    pub board_id: i64,
    /// The starting index of the returned sprints.
    pub start_at: Option<i64>,
    /// The maximum number of sprints to return per page.
    pub max_results: Option<i32>,
    /// Filters results to sprints in specified states (comma-separated).
    pub state: Option<String>,
}

/// Parameters for creating a sprint.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSprintParams {
    /// The name of the sprint.
    pub name: String,
    /// The ID of the board the sprint belongs to.
    pub origin_board_id: i64,
    /// The start date of the sprint (ISO 8601).
    pub start_date: Option<String>,
    /// The end date of the sprint (ISO 8601).
    pub end_date: Option<String>,
    /// The goal of the sprint.
    pub goal: Option<String>,
}

/// Parameters for getting a specific sprint.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetSprintParams {
    /// The ID of the sprint.
    pub sprint_id: i64,
}

/// Parameters for updating a sprint (including starting/closing).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSprintParams {
    /// The ID of the sprint.
    pub sprint_id: i64,
    /// The name of the sprint.
    pub name: Option<String>,
    /// The start date of the sprint (ISO 8601).
    pub start_date: Option<String>,
    /// The end date of the sprint (ISO 8601).
    pub end_date: Option<String>,
    /// The state of the sprint (active, closed).
    pub state: Option<String>,
    /// The goal of the sprint.
    pub goal: Option<String>,
}

/// Parameters for deleting a sprint.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSprintParams {
    /// The ID of the sprint.
    pub sprint_id: i64,
}

/// Paged response for sprints.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanSprint {
    pub max_results: i32,
    pub start_at: i64,
    pub total: Option<i64>,
    pub is_last: bool,
    pub values: Vec<Sprint>,
}

// --- Backlog & Issues ---

/// Paged response for issues (Agile).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanIssue {
    pub expand: Option<String>,
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub issues: Vec<Issue>,
}

/// Parameters for getting issues on a board (backlog or otherwise).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBoardIssuesParams {
    /// The ID of the board.
    pub board_id: i64,
    /// The starting index of the returned issues.
    pub start_at: Option<i64>,
    /// The maximum number of issues to return per page.
    pub max_results: Option<i32>,
    /// JQL query to filter issues.
    pub jql: Option<String>,
    /// Whether to validate the JQL query.
    pub validate_query: Option<bool>,
    /// The fields to return for each issue.
    pub fields: Option<Vec<String>>,
    /// Expansion options.
    pub expand: Option<String>,
}

/// Parameters for getting issues in the backlog (Agile API sometimes separates this, or uses GetBoardIssues).
/// Jira Cloud Agile API has `GET /rest/agile/1.0/board/{boardId}/backlog`.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetBoardBacklogParams {
    /// The ID of the board.
    pub board_id: i64,
    /// The starting index of the returned issues.
    pub start_at: Option<i64>,
    /// The maximum number of issues to return per page.
    pub max_results: Option<i32>,
    /// JQL query to filter issues.
    pub jql: Option<String>,
    /// Whether to validate the JQL query.
    pub validate_query: Option<bool>,
    /// The fields to return for each issue.
    pub fields: Option<Vec<String>>,
    /// Expansion options.
    pub expand: Option<String>,
}

/// Parameters for moving issues to a sprint.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveIssuesToSprintParams {
    /// The ID of the sprint.
    pub sprint_id: i64,
    /// The keys or IDs of the issues to move.
    pub issues: Vec<String>,
}

/// Parameters for moving issues to the backlog.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveIssuesToBacklogParams {
    /// The keys or IDs of the issues to move.
    pub issues: Vec<String>,
}
