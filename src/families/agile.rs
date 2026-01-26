use super::JsonValue;
use crate::families::issue::Issue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Agile Queries (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AgileQueryResource {
    Board,
    Sprint,
    Backlog,
    Issues,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgileQueryParams {
    /// Resource type: "board", "sprint", "backlog", "issues"
    pub resource: AgileQueryResource,

    /// Board ID (required for most operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<i64>,

    /// Sprint ID (required when resource is "sprint")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprint_id: Option<i64>,

    /// Filters as JSON (varies by resource type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<JsonValue>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Maximum results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

// ============================================================================
// PUBLIC API - Sprint Management (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgileSprintManageParams {
    /// Operation: "create", "update", "delete", "start", "close"
    pub operation: String,

    /// Sprint ID (required for update/delete/start/close, omit for create)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprint_id: Option<i64>,

    /// Board ID (required for create, optional for others)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<i64>,

    /// Operation payload (structure varies by operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

// ============================================================================
// PUBLIC API - Move Issues (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgileMoveIssuesParams {
    /// Destination: "sprint" or "backlog"
    pub destination: String,

    /// Destination sprint ID (required when destination is "sprint", omit for backlog)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<i64>,

    /// Issue keys or IDs to move
    pub issues: Vec<String>,
}

// ============================================================================
// PUBLIC API - Sprint Analysis (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgileSprintAnalyzeParams {
    /// Sprint ID to analyze (omit to analyze active sprint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprint_id: Option<i64>,

    /// Board ID (required if sprint_id is omitted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<i64>,

    /// Metrics to include: "velocity", "unestimated", "blocked", "capacity", "completion"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<String>>,
}

// ============================================================================
// INTERNAL DTOs - Board Query Filters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoardFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key_or_id: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Sprint Query Filters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SprintFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Issues Query Filters
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssuesFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jql: Option<String>,
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_query: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
}

// ============================================================================
// INTERNAL DTOs - Sprint Create/Update Data
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SprintData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_board_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

// ============================================================================
// RESPONSE TYPES - Boards
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: i64,
    #[serde(rename = "self")]
    pub self_link: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanBoard {
    pub max_results: i32,
    pub start_at: i64,
    pub total: Option<i64>,
    pub is_last: bool,
    pub values: Vec<Board>,
}

// ============================================================================
// RESPONSE TYPES - Sprints
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    pub id: i64,
    #[serde(rename = "self")]
    pub self_link: String,
    pub state: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_board_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanSprint {
    pub max_results: i32,
    pub start_at: i64,
    pub total: Option<i64>,
    pub is_last: bool,
    pub values: Vec<Sprint>,
}

// ============================================================================
// RESPONSE TYPES - Issues
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanIssue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
    pub start_at: i64,
    pub max_results: i32,
    pub total: i64,
    pub issues: Vec<Issue>,
}

// ============================================================================
// RESPONSE TYPES - Sprint Analysis
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SprintAnalysis {
    pub sprint_id: i64,
    pub sprint_name: String,
    pub sprint_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unestimated_issues: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_issues: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_percentage: Option<f64>,
}
