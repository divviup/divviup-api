use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub name: String,

    #[serde(with = "::time::serde::iso8601")]
    pub created_at: OffsetDateTime,

    #[serde(with = "::time::serde::iso8601")]
    pub updated_at: OffsetDateTime,

    pub admin: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::Memberships")]
    Memberships,
    #[sea_orm(has_many = "super::Tasks")]
    Tasks,
    #[sea_orm(has_many = "super::Aggregators")]
    Aggregators,
    #[sea_orm(has_many = "super::ApiTokens")]
    ApiTokens,
}

impl Related<super::Memberships> for Entity {
    fn to() -> RelationDef {
        Relation::Memberships.def()
    }
}

impl Related<super::Aggregators> for Entity {
    fn to() -> RelationDef {
        Relation::Aggregators.def()
    }
}

impl Related<super::Tasks> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

impl Related<super::ApiTokens> for Entity {
    fn to() -> RelationDef {
        Relation::ApiTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct NewAccount {
    #[validate(required, length(min = 3, max = 100))]
    name: Option<String>,
}

impl Model {
    pub fn build(name: String) -> Result<ActiveModel, ValidationErrors> {
        NewAccount { name: Some(name) }.build()
    }
}

impl NewAccount {
    pub fn build(self) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;
        Ok(ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            name: ActiveValue::Set(self.name.unwrap()),
            created_at: ActiveValue::Set(TimeDateTimeWithTimeZone::now_utc()),
            updated_at: ActiveValue::Set(TimeDateTimeWithTimeZone::now_utc()),
            admin: ActiveValue::Set(false),
        })
    }
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UpdateAccount {
    #[validate(required, length(min = 3, max = 100))]
    name: Option<String>,
}

impl UpdateAccount {
    pub fn build(self, account: Model) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;
        let mut am = account.into_active_model();
        am.name = ActiveValue::Set(self.name.unwrap());
        am.updated_at = ActiveValue::Set(TimeDateTimeWithTimeZone::now_utc());
        Ok(am)
    }
}
