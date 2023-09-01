use crate::HpkeConfigContents;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct HpkeConfig {
    pub id: Uuid,
    pub contents: HpkeConfigContents,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
}
