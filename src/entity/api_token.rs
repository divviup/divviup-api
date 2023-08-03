use super::{Account, AccountColumn, AccountRelation, Accounts, Memberships};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::random;
use sea_orm::{
    ActiveModelBehavior, ActiveValue, ConnectionTrait, DeriveEntityModel, DerivePrimaryKey,
    DeriveRelation, EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, Related, RelationDef,
    RelationTrait,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;
use uuid::Uuid;

const TOKEN_IDENTIFIER: &str = "DUAT";

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

fn encode_token(id: Uuid, token: &[u8; 32]) -> String {
    format!(
        "{TOKEN_IDENTIFIER}{}",
        URL_SAFE_NO_PAD.encode(
            id.as_bytes()
                .iter()
                .chain(token.iter())
                .copied()
                .collect::<Vec<u8>>(),
        )
    )
}

fn decode_token(token: &str) -> Option<(Uuid, [u8; 32])> {
    let token = token.strip_prefix(TOKEN_IDENTIFIER)?;
    let bytes: [u8; 48] = URL_SAFE_NO_PAD.decode(token).ok()?.try_into().ok()?;
    let id = Uuid::from_slice(&bytes[0..16]).ok()?;
    Some((id, bytes[16..].try_into().ok()?))
}

impl Model {
    pub fn build(account: &Account) -> (ActiveModel, String) {
        let token: [u8; 32] = random();
        let id = Uuid::new_v4();
        let token_hash = Sha256::digest(token).to_vec();
        (
            Self {
                id,
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
            encode_token(id, &token),
        )
    }

    pub fn mark_last_used(self) -> ActiveModel {
        let mut api_token = self.into_active_model();
        api_token.last_used_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        api_token
    }

    pub fn tombstone(self) -> ActiveModel {
        let mut api_token = self.into_active_model();
        api_token.deleted_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        api_token.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        api_token
    }

    pub fn is_tombstoned(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl Entity {
    pub async fn load_and_check(
        token: &str,
        db: &impl ConnectionTrait,
    ) -> Option<(Model, Account)> {
        let (id, token) = decode_token(token)?;
        let sha = Sha256::digest(token);
        let (api_token, account) = Self::find_by_id(id)
            .find_also_related(Accounts)
            .one(db)
            .await
            .ok()??;
        if api_token.token_hash.ct_eq(&sha).into() {
            Some((api_token, account?))
        } else {
            None
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateApiToken {
    name: Option<String>,
}
impl UpdateApiToken {
    pub fn build(self, model: Model) -> Result<ActiveModel, crate::handler::Error> {
        let mut api_token = model.into_active_model();
        api_token.name = ActiveValue::Set(match self.name {
            Some(token) if token.is_empty() => None,
            token => token,
        });

        api_token.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        Ok(api_token)
    }
}
