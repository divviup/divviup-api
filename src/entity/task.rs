use crate::{
    client::TaskResponse,
    entity::{account, membership, Account},
    handler::Error,
};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use time::OffsetDateTime;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub partner: String,
    pub vdaf: Json,
    pub min_batch_size: i64,
    pub max_batch_size: Option<i64>,
    pub is_leader: bool,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub time_precision_seconds: i32,
    pub report_count: i32,
    pub aggregate_collection_count: i32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<OffsetDateTime>,
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

impl Related<membership::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Membership.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Task.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Deserialize, Validate, Debug, Clone)]
pub struct NewTask {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,

    #[validate(required)]
    pub partner: Option<String>,

    #[validate(required_nested)]
    pub vdaf: Option<Vdaf>,

    #[validate(required, range(min = 100))]
    pub min_batch_size: Option<i64>,

    #[validate(range(min = 0))]
    pub max_batch_size: Option<i64>,

    #[validate(required)]
    pub is_leader: Option<bool>,

    #[validate(custom = "in_the_future")]
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<TimeDateTimeWithTimeZone>,

    #[validate(
        required,
        range(
            min = 60,
            max = 2592000,
            message = "must be between 1 minute and 4 weeks"
        )
    )]
    pub time_precision_seconds: Option<i32>,

    #[validate(required_nested)]
    pub hpke_config: Option<HpkeConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct HpkeConfig {
    #[validate(required)]
    pub id: Option<u8>,

    #[validate(required)]
    pub kem_id: Option<u8>,

    #[validate(required)]
    pub kdf_id: Option<u8>,

    #[validate(required)]
    pub aead_id: Option<u8>,

    #[validate(required)]
    pub public_key: Option<String>,
}

fn in_the_future(time: &TimeDateTimeWithTimeZone) -> Result<(), ValidationError> {
    if time < &TimeDateTimeWithTimeZone::now_utc() {
        return Err(ValidationError::new("past"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct Histogram {
    #[validate(required, custom = "sorted", custom = "unique")]
    pub buckets: Option<Vec<i32>>,
}

fn sorted(buckets: &Vec<i32>) -> Result<(), ValidationError> {
    let mut buckets_cloned = buckets.clone();
    buckets_cloned.sort_unstable();
    if &buckets_cloned == buckets {
        Ok(())
    } else {
        Err(ValidationError::new("sorted"))
    }
}

fn unique(buckets: &Vec<i32>) -> Result<(), ValidationError> {
    if buckets.iter().collect::<HashSet<_>>().len() == buckets.len() {
        Ok(())
    } else {
        Err(ValidationError::new("unique"))
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy)]
pub struct Sum {
    #[validate(required)]
    pub bits: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Vdaf {
    #[serde(rename = "count")]
    Count,

    #[serde(rename = "histogram")]
    Histogram(Histogram),

    #[serde(rename = "sum")]
    Sum(Sum), // 128 is ceiling

    #[serde(other)]
    Unrecognized,
}

impl Validate for Vdaf {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Vdaf::Count => Ok(()),
            Vdaf::Histogram(h) => h.validate(),
            Vdaf::Sum(s) => s.validate(),
            Vdaf::Unrecognized => {
                let mut errors = ValidationErrors::new();
                errors.add("type", ValidationError::new("unknown"));
                Err(errors)
            }
        }
    }
}

//add query type
//max batch query count

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateTask {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
}
impl UpdateTask {
    pub fn build(self, model: Model) -> Result<ActiveModel, Error> {
        self.validate()?;
        let mut am = model.into_active_model();
        am.name = Set(self.name.unwrap());
        am.updated_at = Set(TimeDateTimeWithTimeZone::now_utc());
        Ok(am)
    }
}

pub fn build_task(mut task: NewTask, api_response: TaskResponse, account: &Account) -> ActiveModel {
    ActiveModel {
        id: Set(api_response.task_id),
        account_id: Set(account.id),
        name: Set(task.name.take().unwrap()),
        partner: Set("".into()),
        vdaf: Set(serde_json::to_value(Vdaf::from(api_response.vdaf)).unwrap()),
        min_batch_size: Set(api_response.min_batch_size),
        max_batch_size: Set(api_response.query_type.into()),
        is_leader: Set(api_response.role.is_leader()),
        created_at: Set(OffsetDateTime::now_utc()),
        updated_at: Set(OffsetDateTime::now_utc()),
        time_precision_seconds: Set(api_response.time_precision),
        report_count: Set(0),
        aggregate_collection_count: Set(0),
        expiration: Set(task.expiration.take()),
    }
}
