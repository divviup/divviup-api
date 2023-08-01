use crate::{
    entity::*,
    queue::job::{EnqueueJob, Job, JobError, SharedJobState, V1},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct TaskSync;

const SYNC_PERIOD: Duration = Duration::weeks(1);

impl TaskSync {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        let aggregators = Aggregators::find()
            .filter(AggregatorColumn::IsFirstParty.eq(true)) // eventually we may want to check for a capability
            .all(db)
            .await?;

        for aggregator in aggregators {
            let client = aggregator.client(job_state.http_client.clone());
            for task_id in client.get_task_ids().await? {
                if 0 == Tasks::find_by_id(&task_id).count(db).await? {
                    client.delete_task(&task_id).await?;
                }
            }
        }

        Ok(Some(EnqueueJob::from(*self).scheduled_in(SYNC_PERIOD)))
    }
}

impl From<TaskSync> for Job {
    fn from(value: TaskSync) -> Self {
        Self::V1(V1::TaskSync(value))
    }
}

impl PartialEq<Job> for TaskSync {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::TaskSync(c)) if c == self)
    }
}
impl PartialEq<TaskSync> for Job {
    fn eq(&self, other: &TaskSync) -> bool {
        matches!(self, Job::V1(V1::TaskSync(j)) if j == other)
    }
}
