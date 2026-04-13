use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ProjectSearchArgs {
    #[schemars(
        description = "The index of the first item to return in a page of results (page offset)."
    )]
    pub start_at: Option<u32>,

    #[schemars(
        description = "The maximum number of items to return per page. Max: 50."
    )]
    pub max_results: Option<u32>,

    #[schemars(
        description = "A query string used to search properties. The query is matched against the project's name and key."
    )]
    pub query: Option<String>,

    #[schemars(
        description = "The statuses of the projects to return. Acceptable values are 'active', 'archived', 'deleted'."
    )]
    pub status: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ProjectGetMetadataArgs {
    #[schemars(
        description = "The project key or ID to fetch metadata for (e.g., 'SCRUM')."
    )]
    pub project_key: String,
}
