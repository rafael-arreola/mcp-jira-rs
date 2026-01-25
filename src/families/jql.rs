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
