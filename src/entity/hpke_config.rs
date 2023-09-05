use super::codec::Codec;
use crate::entity::{AccountColumn, Accounts, Tasks};
use base64::{engine::general_purpose::STANDARD, Engine};
use janus_messages::codec::Decode;
use sea_orm::{
    ActiveModelBehavior, ActiveValue, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors};

#[derive(Clone, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Debug)]
#[sea_orm(table_name = "hpke_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub contents: Codec<janus_messages::HpkeConfig>,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "::time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub name: Option<String>,
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

    pub fn contents(&self) -> &janus_messages::HpkeConfig {
        &self.contents
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
pub struct NewHpkeConfig {
    contents: Option<String>,
    name: Option<String>,
}

impl NewHpkeConfig {
    pub fn build(self, account: &crate::entity::Account) -> Result<ActiveModel, crate::Error> {
        let string = self.contents.ok_or_else(|| {
            let mut validation_errors = ValidationErrors::new();
            validation_errors.add("contents", ValidationError::new("base64"));
            validation_errors
        })?;

        let bytes = STANDARD.decode(string).map_err(|_| {
            let mut validation_errors = ValidationErrors::new();
            validation_errors.add("contents", ValidationError::new("base64"));
            validation_errors
        })?;

        let hpke_config = janus_messages::HpkeConfig::get_decoded(&bytes).map_err(|e| {
            let mut validation_errors = ValidationErrors::new();
            validation_errors.add(
                "contents",
                ValidationError {
                    code: "hpke_config".into(),
                    message: Some(e.to_string().into()),
                    params: Default::default(),
                },
            );
            validation_errors
        })?;

        Ok(Model {
            id: Uuid::new_v4(),
            account_id: account.id,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
            contents: hpke_config.into(),
            name: self.name,
        }
        .into_active_model())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHpkeConfig {
    name: Option<String>,
}

impl UpdateHpkeConfig {
    pub fn build(self, hpke_config: Model) -> Result<ActiveModel, crate::Error> {
        let mut hpke_config = hpke_config.into_active_model();
        hpke_config.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        hpke_config.name = ActiveValue::Set(self.name);
        Ok(hpke_config)
    }
}
