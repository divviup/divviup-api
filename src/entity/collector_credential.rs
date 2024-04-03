use super::codec::Codec;
use crate::entity::{AccountColumn, Accounts, Tasks};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use janus_messages::codec::Decode;
use rand::random;
use sea_orm::{
    ActiveModelBehavior, ActiveValue, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors};

#[derive(Clone, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Debug)]
#[sea_orm(table_name = "collector_credential")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub hpke_config: Codec<janus_messages::HpkeConfig>,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
    pub token_hash: Option<String>,
    #[sea_orm(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl Model {
    pub fn tombstone(self) -> ActiveModel {
        let mut api_token = self.into_active_model();
        api_token.deleted_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
        api_token.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        api_token
    }

    pub fn is_tombstoned(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn hpke_config(&self) -> &janus_messages::HpkeConfig {
        &self.hpke_config
    }

    pub fn new_token() -> (String, String) {
        let token_bytes: [u8; 16] = random();
        // The canonical form of the auth token is the Base64URL unpadded encoding of the bytes.
        let token = URL_SAFE_NO_PAD.encode(token_bytes);
        let token_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()));
        (token, token_hash)
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

    #[sea_orm(has_many = "Tasks")]
    Tasks,
}

impl Related<Tasks> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

impl Related<Accounts> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCollectorCredential {
    hpke_config: Option<String>,
    name: Option<String>,
}

impl NewCollectorCredential {
    pub fn build(
        self,
        account: &crate::entity::Account,
    ) -> Result<(ActiveModel, String), crate::Error> {
        let string = self.hpke_config.ok_or_else(|| {
            let mut validation_errors = ValidationErrors::new();
            validation_errors.add("hpke_config", ValidationError::new("base64"));
            validation_errors
        })?;

        let bytes = STANDARD.decode(string).map_err(|_| {
            let mut validation_errors = ValidationErrors::new();
            validation_errors.add("hpke_config", ValidationError::new("base64"));
            validation_errors
        })?;

        let collector_credential =
            janus_messages::HpkeConfig::get_decoded(&bytes).map_err(|e| {
                let mut validation_errors = ValidationErrors::new();
                validation_errors.add(
                    "hpke_config",
                    ValidationError {
                        code: "collector_credential".into(),
                        message: Some(e.to_string().into()),
                        params: Default::default(),
                    },
                );
                validation_errors
            })?;

        let (token, token_hash) = Model::new_token();
        Ok((
            Model {
                id: Uuid::new_v4(),
                account_id: account.id,
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                deleted_at: None,
                hpke_config: Codec::new(collector_credential)?,
                name: self.name,
                token_hash: Some(token_hash),
                token: None,
            }
            .into_active_model(),
            token,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollectorCredential {
    name: Option<String>,
}

impl UpdateCollectorCredential {
    pub fn build(self, collector_credential: Model) -> Result<ActiveModel, crate::Error> {
        let mut collector_credential = collector_credential.into_active_model();
        collector_credential.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        collector_credential.name = ActiveValue::Set(self.name);
        Ok(collector_credential)
    }
}
