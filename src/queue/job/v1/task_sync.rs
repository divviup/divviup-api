use crate::{
    entity::*,
    queue::job::{EnqueueJob, Job, JobError, SharedJobState, V1},
};
use futures_lite::StreamExt;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
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
            let client = aggregator
                .client(job_state.http_client.clone(), &job_state.crypter)
                .map_err(|e| JobError::ClientOther(e.to_string()))?;
            while let Some(task_from_aggregator) = client.task_stream().next().await.transpose()? {
                let task_id = task_from_aggregator.task_id.to_string();
                if let Some(_task_from_db) = Tasks::find_by_id(&task_id).one(db).await? {
                    // TODO: confirm that the task matches
                } else {
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
