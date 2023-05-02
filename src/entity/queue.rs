use crate::{
    json_newtype,
    queue::{Job, JobError},
};
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum JobResult {
    Complete,
    Child(Uuid),
    Error(JobError),
}

json_newtype!(JobResult);
json_newtype!(Job);

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub scheduled_at: Option<OffsetDateTime>,
    pub failure_count: i32,
    pub status: JobStatus,
    pub job: Job,
    pub result: Option<JobResult>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum JobStatus {
    #[sea_orm(num_value = 0)]
    Pending,
    #[sea_orm(num_value = 1)]
    Success,
    #[sea_orm(num_value = 2)]
    Failed,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Job> for ActiveModel {
    fn from(job: Job) -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            scheduled_at: Set(None),
            failure_count: Set(0),
            status: Set(JobStatus::Pending),
            job: Set(job),
            result: Set(None),
            parent_id: Set(None),
        }
    }
}
