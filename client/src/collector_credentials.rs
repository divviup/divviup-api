use crate::HpkeConfig;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CollectorCredential {
    pub id: Uuid,
    pub hpke_config: HpkeConfig,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
    pub token_hash: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}
