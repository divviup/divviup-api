use crate::entity::{account, task};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_email: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<task::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Task.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Membership.def().rev())
    }
}

#[derive(serde::Deserialize, Clone, Debug, Validate)]
pub struct CreateMembership {
    #[validate(required, email)]
    pub user_email: Option<String>,
}

impl CreateMembership {
    pub fn build(self, account: &account::Model) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;
        Ok(ActiveModel {
            id: Set(Uuid::new_v4()),
            account_id: Set(account.id.clone()),
            user_email: Set(self.user_email.unwrap()),
            created_at: Set(TimeDateTimeWithTimeZone::now_utc()),
        })
    }
}

impl ActiveModelBehavior for ActiveModel {}
