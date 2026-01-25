use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details about an issue priority.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Priority {
    /// The ID of the priority.
    pub id: String,
    /// The name of the priority.
    pub name: String,
    /// The description of the priority.
    pub description: Option<String>,
    /// The URL of the priority's icon.
    pub icon_url: Option<String>,
    /// The color used to indicate the priority.
    pub status_color: Option<String>,
    /// Whether the priority is the default priority.
    pub is_default: Option<bool>,
    /// The URL of the priority.
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Details about an issue resolution.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Resolution {
    /// The ID of the resolution.
    pub id: String,
    /// The name of the resolution.
    pub name: String,
    /// The description of the resolution.
    pub description: Option<String>,
    /// Whether the resolution is the default resolution.
    pub is_default: Option<bool>,
    /// The URL of the resolution.
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Parameters for getting labels.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetLabelsParams {
    /// The index of the first item to return in a page of results (0-based).
    pub start_at: Option<i64>,
    /// The maximum number of items to return per page.
    pub max_results: Option<i32>,
}

/// Paged response for strings (labels).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanString {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<String>,
}

/// Parameters for searching priorities.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchPrioritiesParams {
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
    pub id: Option<Vec<String>>,
    pub project_id: Option<Vec<String>>,
    pub priority_name: Option<String>,
    pub only_default: Option<bool>,
    pub expand: Option<String>,
}

/// Paged response for priorities.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanPriority {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<Priority>,
}

/// Parameters for searching resolutions.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResolutionsParams {
    pub start_at: Option<i64>,
    pub max_results: Option<i32>,
    pub id: Option<Vec<String>>,
    pub only_default: Option<bool>,
}

/// Paged response for resolutions.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBeanResolution {
    pub is_last: bool,
    pub max_results: i32,
    pub start_at: i64,
    pub total: i64,
    pub values: Vec<Resolution>,
}
