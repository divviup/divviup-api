use super::{Account, AccountColumn, AccountRelation, Accounts, Memberships};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sea_orm::{
    ActiveModelBehavior, DeriveEntityModel, DerivePrimaryKey, DeriveRelation, EntityTrait,
    EnumIter, IntoActiveModel, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "api_token")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    #[serde(with = "url_safe_base64")]
    pub token_hash: Vec<u8>,
    #[serde(with = "::time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::iso8601::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(default, with = "::time::serde::iso8601::option")]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
    #[sea_orm(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

mod url_safe_base64 {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use serde::{
        de::{Error, Unexpected, Visitor},
        Deserializer, Serializer,
    };

    pub fn serialize<S: Serializer>(
        token_hash: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&URL_SAFE_NO_PAD.encode(token_hash))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        struct Base64Visitor;
        impl<'de> Visitor<'de> for Base64Visitor {
            type Value = Vec<u8>;
            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "base64")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                URL_SAFE_NO_PAD
                    .decode(v)
                    .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(Base64Visitor)
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiToken")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("account_id", &self.account_id)
            .field("token_hash", &URL_SAFE_NO_PAD.encode(&self.token_hash))
            .field("created_at", &self.created_at)
            .field("deleted_at", &self.deleted_at)
            .field("last_used_at", &self.last_used_at)
            .field("updated_at", &self.deleted_at)
            .finish()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Accounts",
        from = "Column::AccountId",
        to = "AccountColumn::Id"
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
        Some(AccountRelation::ApiTokens.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn build(account: &Account) -> (ActiveModel, String) {
        let mut token = [0u8; 32];
        rand::thread_rng().fill(&mut token);
        let token_hash = Sha256::digest(token).to_vec();
        (
            Self {
                id: Uuid::new_v4(),
                account_id: account.id,
                token_hash,
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                deleted_at: None,
                token: None,
                last_used_at: None,
                name: None,
            }
            .into_active_model(),
            URL_SAFE_NO_PAD.encode(token),
        )
    }

    pub fn tombstone(self) -> ActiveModel {
        let mut api_token = self.into_active_model();
        api_token.deleted_at = sea_orm::ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        api_token.updated_at = sea_orm::ActiveValue::Set(OffsetDateTime::now_utc());
        api_token
    }

    pub fn is_tombstoned(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateApiToken {
    name: Option<String>,
}
impl UpdateApiToken {
    pub fn build(self, model: Model) -> Result<ActiveModel, crate::handler::Error> {
        let mut api_token = model.into_active_model();
        api_token.name = sea_orm::ActiveValue::Set(match self.name {
            Some(token) if token.is_empty() => None,
            token => token,
        });

        api_token.updated_at = sea_orm::ActiveValue::Set(OffsetDateTime::now_utc());
        Ok(api_token)
    }
}
