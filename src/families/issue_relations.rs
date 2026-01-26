use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// PUBLIC API - Relations Management (Exposed to MCP)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueRelationsParams {
    /// Relation type: "attachment" or "link"
    pub relation_type: String,

    /// Operation: "get", "create", "delete"
    pub operation: String,

    /// Issue ID or key
    pub issue_id_or_key: String,

    /// Relation ID (required for delete, omit for get all or create)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_id: Option<String>,

    /// Operation payload (required for create, structure varies by relation type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

// ============================================================================
// INTERNAL DTOs - Link Operations
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateIssueLinkData {
    pub r#type: IssueLinkType,
    pub inward_issue: IssueRef,
    pub outward_issue: IssueRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<LinkComment>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueLinkType {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueRef {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LinkComment {
    pub body: JsonValue,
}

// ============================================================================
// RESPONSE TYPES - Attachments
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AttachmentUser>,
    pub created: String,
    pub size: i64,
    pub mime_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUser {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

// ============================================================================
// RESPONSE TYPES - Links
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueLink {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub r#type: IssueLinkTypeResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inward_issue: Option<LinkedIssue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outward_issue: Option<LinkedIssue>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IssueLinkTypeResponse {
    pub id: String,
    pub name: String,
    pub inward: String,
    pub outward: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkedIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<JsonValue>,
}
