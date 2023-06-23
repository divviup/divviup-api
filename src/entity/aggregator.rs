use crate::clients::AggregatorClient;

use super::{account, membership, url::Url};
use sea_orm::{entity::prelude::*, IntoActiveModel, Set};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use time::OffsetDateTime;

mod update_aggregator;
pub use update_aggregator::UpdateAggregator;

mod new_aggregator;
pub use new_aggregator::NewAggregator;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "aggregator")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    // an account_id of None indicates a shared Aggregator
    pub account_id: Option<Uuid>,
    #[serde(with = "::time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "::time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    // a deleted_at of Some indicates a tombstoned/inactivated Aggregator
    #[serde(default, with = "time::serde::iso8601::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub role: Role,
    pub name: String,
    pub dap_url: Url,
    pub api_url: Url,
    pub is_first_party: bool,
    #[serde(skip)]
    pub bearer_token: String,
}

impl Model {
    pub fn tombstone(self) -> ActiveModel {
        let mut aggregator = self.into_active_model();
        aggregator.updated_at = Set(OffsetDateTime::now_utc());
        aggregator.deleted_at = Set(Some(OffsetDateTime::now_utc()));
        aggregator
    }

    pub fn is_tombstoned(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn is_first_party(&self) -> bool {
        // probably temporary
        matches!(self.dap_url.domain(), Some(domain) if domain.ends_with("divviup.org"))
    }

    pub fn client(&self, http_client: trillium_client::Client) -> AggregatorClient {
        AggregatorClient::new(http_client, self.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Role {
    #[sea_orm(num_value = 0)]
    Leader,
    #[sea_orm(num_value = 1)]
    Helper,
    #[sea_orm(num_value = 2)]
    Either,
}
impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        match self {
            Self::Leader => "leader",
            Self::Helper => "helper",
            Self::Either => "either",
        }
    }
}

#[derive(Debug)]
pub struct UnrecognizedRole(String);
impl Display for UnrecognizedRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} was not a recognized role option", self.0))
    }
}
impl Error for UnrecognizedRole {}
impl FromStr for Role {
    type Err = UnrecognizedRole;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "leader" => Ok(Self::Leader),
            "helper" => Ok(Self::Helper),
            "either" => Ok(Self::Either),
            unrecognized => Err(UnrecognizedRole(unrecognized.to_string())),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Account,
}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<membership::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Membership.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Aggregator.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
