use super::JsonValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details about a user.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub struct User {
    /// The account ID of the user.
    pub account_id: String,
    /// The account type of the user.
    pub account_type: Option<String>,
    /// The display name of the user.
    pub display_name: Option<String>,
    /// The email address of the user.
    pub email_address: Option<String>,
    /// Whether the user is active.
    pub active: Option<bool>,
    /// The time zone of the user.
    pub time_zone: Option<String>,
    /// The locale of the user.
    pub locale: Option<String>,
    /// The avatars of the user.
    pub avatar_urls: Option<JsonValue>,
}