use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details of an attachment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct Attachment {
    /// The ID of the attachment.
    pub id: String,
    /// The URL of the attachment details.
    #[serde(rename = "self")]
    pub self_link: String,
    /// The name of the attachment file.
    pub filename: String,
    /// The author of the attachment.
    pub author: Option<AttachmentUser>,
    /// The datetime the attachment was created.
    pub created: String,
    /// The size of the attachment.
    pub size: i64,
    /// The MIME type of the attachment.
    pub mime_type: String,
    /// The content of the attachment.
    pub content: String,
    /// The thumbnail of the attachment.
    pub thumbnail: Option<String>,
}

/// Details about a user in the context of an attachment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct AttachmentUser {
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub active: Option<bool>,
    pub avatar_urls: Option<JsonValue>,
}

// --- Parameters ---

/// Parameters for getting attachments for an issue.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetAttachmentsParams {
    /// The ID or key of the issue.
    pub issue_id_or_key: String,
}

/// Parameters for deleting an attachment.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAttachmentParams {
    /// The ID of the attachment.
    pub attachment_id: String,
}
