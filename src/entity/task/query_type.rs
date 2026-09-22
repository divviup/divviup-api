use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "query_type")]
pub enum QueryType {
    #[sea_orm(string_value = "TIME_INTERVAL")]
    TimeInterval,
    #[sea_orm(string_value = "FIXED_SIZE")]
    FixedSize,
}
