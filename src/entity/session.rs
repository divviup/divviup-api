use sea_orm::{
    ActiveModelBehavior, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EnumIter, PrimaryKeyTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: String,
    pub expiry: Option<OffsetDateTime>,
    pub data: Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
