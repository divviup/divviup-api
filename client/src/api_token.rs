use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiToken {
    pub id: Uuid,
    pub account_id: Uuid,
    pub token_hash: String,
    #[serde(with = "::time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::iso8601::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(default, with = "::time::serde::iso8601::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
    pub token: Option<String>,
}
