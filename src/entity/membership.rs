use crate::{
    entity::{Account, AccountColumn, AccountRelation, Accounts, Aggregators, Tasks},
    User,
};
use sea_orm::{
    ActiveModelBehavior, ColumnTrait, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, QueryFilter, Related, RelationDef,
    RelationTrait, Select,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "membership")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_email: String,
    #[serde(with = "::time::serde::rfc3339")]
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

impl Entity {
    pub fn for_user(user: &User) -> Select<Self> {
        Self::find().filter(Column::UserEmail.eq(&user.email))
    }
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
            created_at: OffsetDateTime::now_utc(),
        }
        .into_active_model())
    }
}

impl ActiveModelBehavior for ActiveModel {}
