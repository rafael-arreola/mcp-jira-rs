use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct IssueLinkType {
    pub name: Option<String>,
    pub inward: Option<String>,
    pub outward: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct LinkIssueRef {
    pub key: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueLinkParams {
    #[serde(rename = "type")]
    pub link_type: IssueLinkType,
    pub inward_issue: LinkIssueRef,
    pub outward_issue: LinkIssueRef,
    pub comment: Option<super::issue_comment::Comment>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteIssueLinkParams {
    pub link_id: String,
}
