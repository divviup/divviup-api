use crate::{
    json_newtype,
    queue::{EnqueueJob, Job, JobError},
};
use sea_orm::{
    entity::prelude::*,
    sea_query::{self, all, any, LockBehavior, LockType},
    DatabaseTransaction, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use time::OffsetDateTime;

json_newtype!(JobError);
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
    pub error_message: Option<JobError>,
    pub parent_id: Option<Uuid>,
    pub child_id: Option<Uuid>,
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
            error_message: Set(None),
            parent_id: Set(None),
            child_id: Set(None),
        }
    }
}

impl From<EnqueueJob> for ActiveModel {
    fn from(EnqueueJob { job, scheduled }: EnqueueJob) -> Self {
        let mut am = Self::from(job);
        am.scheduled_at = Set(scheduled);
        am
    }
}

impl Entity {
    pub async fn next(tx: &DatabaseTransaction) -> Result<Option<Model>, DbErr> {
        use Column::*;
        let mut select = Entity::find()
            .filter(all![
                Status.eq(JobStatus::Pending),
                any![
                    ScheduledAt.is_null(),
                    ScheduledAt.lt(OffsetDateTime::now_utc())
                ]
            ])
            .order_by_asc(CreatedAt)
            .limit(1);

        QuerySelect::query(&mut select)
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked);

        select.one(tx).await
    }
}
