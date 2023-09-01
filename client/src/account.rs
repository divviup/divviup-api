use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub admin: bool,
}
