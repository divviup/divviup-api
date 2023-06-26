use crate::entity::{Account, AccountColumn, AccountRelation, Accounts, Aggregators, Tasks};
use sea_orm::{prelude::*, IntoActiveModel};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_email: String,
    #[serde(with = "::time::serde::iso8601")]
    pub created_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Accounts",
        from = "Column::AccountId",
        to = "AccountColumn::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
}

impl Related<Accounts> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<Tasks> for Entity {
    fn to() -> RelationDef {
        AccountRelation::Tasks.def()
    }

    fn via() -> Option<RelationDef> {
        Some(AccountRelation::Memberships.def().rev())
    }
}

impl Related<Aggregators> for Entity {
    fn to() -> RelationDef {
        AccountRelation::Aggregators.def()
    }

    fn via() -> Option<RelationDef> {
        Some(AccountRelation::Memberships.def().rev())
    }
}

impl Model {
    pub fn build(email: String, account: &Account) -> Result<ActiveModel, ValidationErrors> {
        CreateMembership {
            user_email: Some(email),
        }
        .build(account)
    }
}

#[derive(serde::Deserialize, Clone, Debug, Validate)]
pub struct CreateMembership {
    #[validate(required, email)]
    pub user_email: Option<String>,
}

impl CreateMembership {
    pub fn build(self, account: &Account) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;
        Ok(Model {
            id: Uuid::new_v4(),
            account_id: account.id,
            user_email: self.user_email.unwrap(),
            created_at: TimeDateTimeWithTimeZone::now_utc(),
        }
        .into_active_model())
    }
}

impl ActiveModelBehavior for ActiveModel {}
