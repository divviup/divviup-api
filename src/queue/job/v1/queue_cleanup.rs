use crate::{
    entity::queue::{Column as QueueColumn, Entity as QueueEntity, JobStatus},
    queue::job::{v1::V1, EnqueueJob, Job, JobError, SharedJobState},
};
use sea_orm::{
    sea_query::{self, all, Expr},
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

const CLEANUP_PERIOD: Duration = Duration::minutes(60);
const RETENTION_PERIOD: Duration = Duration::weeks(2);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct QueueCleanup;

impl QueueCleanup {
    pub async fn perform(
        &mut self,
        _job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        QueueEntity::delete_many()
            .filter(all![
                Expr::cust_with_expr("job->>'type' = $1", "QueueCleanup"),
                QueueColumn::ScheduledAt.gt(OffsetDateTime::now_utc()),
            ])
            .exec(db)
            .await?;

        QueueEntity::delete_many()
            .filter(all![
                QueueColumn::Status.eq(JobStatus::Success),
                QueueColumn::UpdatedAt.lt(OffsetDateTime::now_utc() - RETENTION_PERIOD)
            ])
            .exec(db)
            .await?;

        Ok(Some(
            EnqueueJob::from(QueueCleanup).scheduled_in(CLEANUP_PERIOD),
        ))
    }
}

impl From<QueueCleanup> for Job {
    fn from(value: QueueCleanup) -> Self {
        Self::V1(V1::QueueCleanup(value))
    }
}

impl PartialEq<Job> for QueueCleanup {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::QueueCleanup(c)) if c == self)
    }
}
impl PartialEq<QueueCleanup> for Job {
    fn eq(&self, other: &QueueCleanup) -> bool {
        matches!(self, Job::V1(V1::QueueCleanup(j)) if j == other)
    }
}
