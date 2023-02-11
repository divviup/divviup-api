use std::collections::HashSet;

use crate::{
    entity::{account, membership},
    handler::Error,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub partner: String,
    pub vdaf: Json,
    pub min_batch_size: i64,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: TimeDateTimeWithTimeZone,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: TimeDateTimeWithTimeZone,
    pub time_precision_seconds: i32,
    pub report_count: i32,
    pub aggregate_collection_count: i32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<TimeDateTimeWithTimeZone>,
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

#[derive(Deserialize, Validate, Debug)]
pub struct NewTask {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,

    #[validate(required)]
    pub partner: Option<String>,

    #[validate(required_nested)]
    pub vdaf: Option<Vdaf>,

    #[validate(required, range(min = 100))]
    pub min_batch_size: Option<i64>,

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
}

fn in_the_future(time: &TimeDateTimeWithTimeZone) -> Result<(), ValidationError> {
    if time < &TimeDateTimeWithTimeZone::now_utc() {
        return Err(ValidationError::new("past"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct Histogram {
    #[validate(required, custom = "sorted", custom = "unique")]
    pub buckets: Option<Vec<usize>>,
}

fn sorted(buckets: &Vec<usize>) -> Result<(), ValidationError> {
    let mut buckets_cloned = buckets.clone();
    buckets_cloned.sort();
    if &buckets_cloned != buckets {
        Err(ValidationError::new("sorted"))
    } else {
        Ok(())
    }
}

fn unique(buckets: &Vec<usize>) -> Result<(), ValidationError> {
    if buckets.iter().collect::<HashSet<_>>().len() != buckets.len() {
        Err(ValidationError::new("unique"))
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct Sum {
    #[validate(required)]
    pub bits: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
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

impl NewTask {
    pub fn build(self, account: account::Model) -> Result<ActiveModel, ValidationErrors> {
        self.validate()?;

        // this is temporary. janus will generate an id
        let mut id = vec![0u8; 8];
        fastrand::Rng::new().fill(&mut id);

        Ok(ActiveModel {
            id: Set(BASE64.encode(id)),
            account_id: Set(account.id),
            name: Set(self.name.unwrap()), //aggregator endpoint and role
            partner: Set(self.partner.unwrap()),
            vdaf: Set(serde_json::to_value(&self.vdaf.unwrap()).unwrap()),
            min_batch_size: Set(self.min_batch_size.unwrap()),
            created_at: Set(TimeDateTimeWithTimeZone::now_utc()),
            updated_at: Set(TimeDateTimeWithTimeZone::now_utc()),
            time_precision_seconds: Set(self.time_precision_seconds.unwrap()),
            report_count: Set(0),
            aggregate_collection_count: Set(0),
            expiration: Set(self.expiration),
        })
    }
}

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
