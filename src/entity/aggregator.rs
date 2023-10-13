mod feature;
mod new_aggregator;
mod protocol;
mod query_type_name;
mod role;
mod update_aggregator;
mod vdaf_name;

use super::{json::Json, url::Url, AccountColumn, AccountRelation, Accounts, Memberships};
use crate::{clients::AggregatorClient, Crypter, Error};
use sea_orm::{
    ActiveModelBehavior, ActiveValue, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub use feature::{Feature, Features};
pub use new_aggregator::NewAggregator;
pub use protocol::{Protocol, UnrecognizedProtocol};
pub use query_type_name::{QueryTypeName, QueryTypeNameSet};
pub use role::{Role, UnrecognizedRole};
pub use update_aggregator::UpdateAggregator;
pub use vdaf_name::{VdafName, VdafNameSet};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "aggregator")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    // an account_id of None indicates a shared Aggregator
    pub account_id: Option<Uuid>,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    // a deleted_at of Some indicates a tombstoned/inactivated Aggregator
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub role: Role,
    pub name: String,
    pub dap_url: Url,
    pub api_url: Url,
    pub is_first_party: bool,
    pub query_types: Json<QueryTypeNameSet>,
    pub vdafs: Json<VdafNameSet>,
    pub protocol: Protocol,
    #[serde(skip)]
    pub encrypted_bearer_token: Vec<u8>,
    pub features: Json<Features>,
}

impl Model {
    pub fn tombstone(self) -> ActiveModel {
        let mut aggregator = self.into_active_model();
        aggregator.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        aggregator.deleted_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        aggregator
    }

    pub fn is_tombstoned(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn client(
        &self,
        http_client: trillium_client::Client,
        crypter: &Crypter,
    ) -> Result<AggregatorClient, Error> {
        Ok(AggregatorClient::new(
            http_client,
            self.clone(),
            &self.bearer_token(crypter)?,
        ))
    }

    pub fn bearer_token(&self, crypter: &Crypter) -> Result<String, Error> {
        let bearer_token_bytes = crypter.decrypt(
            self.api_url.as_ref().as_bytes(),
            &self.encrypted_bearer_token,
        )?;
        String::from_utf8(bearer_token_bytes).map_err(Into::into)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Accounts",
        from = "Column::AccountId",
        to = "AccountColumn::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Account,
}

impl Related<Accounts> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<Memberships> for Entity {
    fn to() -> RelationDef {
        AccountRelation::Memberships.def()
    }

    fn via() -> Option<RelationDef> {
        Some(AccountRelation::Aggregators.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
