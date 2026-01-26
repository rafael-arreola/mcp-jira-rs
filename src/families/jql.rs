use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for parsing and validating JQL queries.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParseJqlQueryParams {
    /// A list of JQL queries to parse.
    pub queries: Vec<String>,
}

/// The result of parsing JQL queries.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedJqlQueries {
    /// The list of parsed queries.
    pub queries: Vec<ParsedJqlQuery>,
}

/// Details of a parsed JQL query.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedJqlQuery {
    /// The original query string.
    pub query: String,
    /// The syntax tree of the query (if valid).
    pub structure: Option<JsonValue>,
    /// A list of validation errors, if any.
    pub errors: Vec<String>,
}

// ============================================================================
// PUBLIC API - Search Parameters (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    /// JQL query string
    pub jql: String,

    /// Fields to return for each issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    /// Expand options (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,

    /// Maximum results per page (default 50, max 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,

    /// Starting index for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>,

    /// Return fields by keys instead of IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_by_keys: Option<bool>,

    /// Properties to return for each issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,

    /// Validate JQL query before execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_query: Option<bool>,
}
