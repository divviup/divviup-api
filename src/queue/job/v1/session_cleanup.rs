use crate::{
    entity::*,
    queue::job::{v1::V1, EnqueueJob, Job, JobError, SharedJobState},
};
use sea_orm::{
    sea_query::{self, all, Expr},
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

const PERIOD: Duration = Duration::minutes(60);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct SessionCleanup;

impl SessionCleanup {
    pub async fn perform(
        &mut self,
        _job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        queue::Entity::delete_many()
            .filter(all![
                Expr::cust_with_expr("job->>'type' = $1", "SessionCleanup"),
                queue::Column::ScheduledAt.gt(OffsetDateTime::now_utc()),
            ])
            .exec(db)
            .await?;

        Sessions::delete_many()
            .filter(SessionColumn::Expiry.lt(OffsetDateTime::now_utc()))
            .exec(db)
            .await?;
        Ok(Some(EnqueueJob::from(SessionCleanup).scheduled_in(PERIOD)))
    }
}

impl From<SessionCleanup> for Job {
    fn from(value: SessionCleanup) -> Self {
        Self::V1(V1::SessionCleanup(value))
    }
}

impl PartialEq<Job> for SessionCleanup {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::SessionCleanup(c)) if c == self)
    }
}
impl PartialEq<SessionCleanup> for Job {
    fn eq(&self, other: &SessionCleanup) -> bool {
        matches!(self, Job::V1(V1::SessionCleanup(j)) if j == other)
    }
}
