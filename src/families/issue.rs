use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Issues ---

/// Details of an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// The ID of the issue.
    pub id: String,
    /// The key of the issue.
    pub key: String,
    /// The URL of the issue details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// Expand to include additional information.
    pub expand: Option<String>,
    /// The fields of the issue.
    pub fields: HashMap<String, serde_json::Value>,
    /// The rendered fields of the issue.
    pub rendered_fields: Option<HashMap<String, String>>,
}

/// Parameters for creating an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueParams {
    /// The fields of the issue.
    pub fields: JsonValue,
    /// Additional updates to perform on the issue.
    pub update: Option<JsonValue>,
}

/// Parameters for getting an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetIssueParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// A list of fields to return for the issue.
    pub fields: Option<Vec<String>>,
    /// Whether to expand the fields.
    pub expand: Option<String>,
    /// Whether to return the issue properties.
    pub properties: Option<Vec<String>>,
    /// Whether to update the issue history.
    pub update_history: Option<bool>,
}

/// Parameters for editing an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EditIssueParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The fields to update.
    pub fields: Option<JsonValue>,
    /// The updates to perform.
    pub update: Option<JsonValue>,
}

/// Parameters for deleting an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteIssueParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// Whether to delete subtasks.
    pub delete_subtasks: Option<String>,
}

/// Parameters for assigning an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssignIssueParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The account ID of the user to assign the issue to.
    pub account_id: String,
}

/// Parameters for transitioning an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitionIssueParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
    /// The transition to perform.
    pub transition: IssueTransition,
    /// The fields to update.
    pub fields: Option<JsonValue>,
    /// The updates to perform.
    pub update: Option<JsonValue>,
}

/// Details of an issue transition.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct IssueTransition {
    /// The ID of the transition.
    pub id: String,
    /// The name of the transition.
    pub name: Option<String>,
}

impl<'de> serde::Deserialize<'de> for IssueTransition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawTransition {
            id: String,
            name: Option<String>,
        }

        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::String(s) => {
                let raw: RawTransition =
                    serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
                Ok(IssueTransition {
                    id: raw.id,
                    name: raw.name,
                })
            }
            _ => {
                let raw: RawTransition =
                    serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                Ok(IssueTransition {
                    id: raw.id,
                    name: raw.name,
                })
            }
        }
    }
}

// --- Search (JQL) ---

/// Parameters for searching issues using JQL (POST - Enhanced Search).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchIssuesPostParams {
    /// The JQL query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jql: Option<String>,
    /// The token for the next page of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    /// The maximum number of items to return per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    /// A list of fields to return for each issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    /// Whether to expand the fields (comma-separated string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<String>,
    /// Whether to return fields by keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_by_keys: Option<bool>,
    /// A list of issue properties to return for each issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,
    /// A list of issue IDs to reconcile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconcile_issues: Option<Vec<i64>>,
}

/// Results of an enhanced search.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Whether this is the last page of results.
    pub is_last: bool,
    /// The list of issues found.
    pub issues: Vec<Issue>,
    /// The token for the next page of results.
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
}
