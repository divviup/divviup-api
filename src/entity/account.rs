use crate::{
    entity::{Aggregators, ApiTokens, HpkeConfigs, Memberships, Tasks},
    PermissionsActor,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, DeriveEntityModel,
    DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait,
    QueryFilter, Related, RelationDef, RelationTrait, Select,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub name: String,

    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,

    pub admin: bool,

    pub intends_to_use_shared_aggregators: Option<bool>,
}

impl Entity {
    pub fn for_actor(actor: &PermissionsActor) -> Select<Self> {
        if actor.is_admin() {
            Self::find()
        } else {
            Self::find().filter(Column::Id.is_in(actor.account_ids()))
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "Memberships")]
    Memberships,
    #[sea_orm(has_many = "Tasks")]
    Tasks,
    #[sea_orm(has_many = "Aggregators")]
    Aggregators,
    #[sea_orm(has_many = "ApiTokens")]
    ApiTokens,
    #[sea_orm(has_many = "HpkeConfigs")]
    HpkeConfigs,
}

impl Related<Memberships> for Entity {
    fn to() -> RelationDef {
        Relation::Memberships.def()
    }
}

impl Related<Aggregators> for Entity {
    fn to() -> RelationDef {
        Relation::Aggregators.def()
    }
}

impl Related<Tasks> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

impl Related<ApiTokens> for Entity {
    fn to() -> RelationDef {
        Relation::ApiTokens.def()
    }
}

impl Related<HpkeConfigs> for Entity {
    fn to() -> RelationDef {
        Relation::HpkeConfigs.def()
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
        Ok(Model {
            id: Uuid::new_v4(),
            name: self.name.unwrap(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            admin: false,
            intends_to_use_shared_aggregators: None,
        }
        .into_active_model())
    }
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UpdateAccount {
    #[validate(length(min = 3, max = 100))]
    name: Option<String>,
    intends_to_use_shared_aggregators: Option<bool>,
}

impl UpdateAccount {
    pub fn build(self, account: Model) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;
        let mut am = account.into_active_model();
        am.name = self.name.map_or(ActiveValue::NotSet, ActiveValue::Set);
        am.intends_to_use_shared_aggregators = self
            .intends_to_use_shared_aggregators
            .map_or(ActiveValue::NotSet, |b| ActiveValue::Set(Some(b)));
        if am.is_changed() {
            am.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        }
        Ok(am)
    }
}
