mod job;
pub use crate::entity::queue::JobStatus;
pub use job::*;

use crate::{
    entity::queue::{ActiveModel, Column, Entity, Model},
    Config, Db,
};
use sea_orm::{
    sea_query::{all, Expr},
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, TransactionTrait,
};
use std::{ops::Range, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{
    task::{JoinHandle, JoinSet},
    time::sleep,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct Queue {
    cancel: CancellationToken,
    db: Db,
    job_state: Arc<SharedJobState>,
}
/*
These configuration variables may eventually be useful to put on Config
*/
const MAX_RETRY: i32 = 5;
const QUEUE_CHECK_INTERVAL: Range<u64> = 60_000..120_000;
const SCHEDULE_RANDOMNESS: Range<u64> = 0..15_000;
const QUEUE_WORKER_COUNT: u8 = 2;

fn reschedule_based_on_failure_count(failure_count: i32) -> Option<OffsetDateTime> {
    if failure_count >= MAX_RETRY {
        None
    } else {
        let duration = Duration::from_millis(
            1000 * 4_u64.pow(failure_count.try_into().unwrap())
                + fastrand::u64(SCHEDULE_RANDOMNESS),
        );
        Some(OffsetDateTime::now_utc() + duration)
    }
}

impl Queue {
    pub fn new(db: &Db, config: &Config, cancel: CancellationToken) -> Self {
        Self {
            cancel,
            db: db.clone(),
            job_state: Arc::new(config.into()),
        }
    }

    pub async fn schedule_recurring_tasks_if_needed(&self) -> Result<(), DbErr> {
        let tx = self.db.begin().await?;

        let session_cleanup_jobs = Entity::find()
            .filter(all![
                Expr::cust_with_expr("job->>'type' = $1", "SessionCleanup"),
                Column::ScheduledAt.gt(OffsetDateTime::now_utc()),
            ])
            .count(&tx)
            .await?;

        if session_cleanup_jobs == 0 {
            Job::from(SessionCleanup).insert(&tx).await?;
        }
        tx.commit().await?;

        let tx = self.db.begin().await?;
        let queue_cleanup_jobs = Entity::find()
            .filter(all![
                Expr::cust_with_expr("job->>'type' = $1", "QueueCleanup"),
                Column::ScheduledAt.gt(OffsetDateTime::now_utc()),
            ])
            .count(&tx)
            .await?;

        if queue_cleanup_jobs == 0 {
            Job::from(QueueCleanup).insert(&tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    // TODO(#2262): use TaskTracker to wait for in-flight jobs during graceful shutdown
    pub async fn perform_one_queue_job(&self) -> Result<Option<Model>, DbErr> {
        let tx = self.db.begin().await?;
        let model = if let Some(queue_item) = Entity::next(&tx).await? {
            let mut queue_item = queue_item.into_active_model();

            let mut job = queue_item.job.take().ok_or_else(|| {
                DbErr::Custom(String::from(
                    r#"Queue item found without a job.
                       We believe this to be unreachable"#,
                ))
            })?;

            let result = job.perform(&self.job_state, &tx).await;
            queue_item.job = ActiveValue::Set(job);

            match result {
                Ok(Some(next_job)) => {
                    queue_item.status = ActiveValue::Set(JobStatus::Success);
                    queue_item.scheduled_at = ActiveValue::Set(None);

                    let mut next_job = ActiveModel::from(next_job);
                    next_job.parent_id = ActiveValue::Set(Some(*queue_item.id.as_ref()));
                    let next_job = next_job.insert(&tx).await?;
                    queue_item.child_id = ActiveValue::Set(Some(next_job.id));
                }

                Ok(None) => {
                    queue_item.scheduled_at = ActiveValue::Set(None);
                    queue_item.status = ActiveValue::Set(JobStatus::Success);
                }

                Err(e) if e.is_retryable() => {
                    queue_item.failure_count =
                        ActiveValue::Set(queue_item.failure_count.as_ref() + 1);
                    let reschedule =
                        reschedule_based_on_failure_count(*queue_item.failure_count.as_ref());
                    queue_item.status = ActiveValue::Set(
                        reschedule.map_or(JobStatus::Failed, |_| JobStatus::Pending),
                    );
                    queue_item.scheduled_at = ActiveValue::Set(reschedule);
                    queue_item.error_message = ActiveValue::Set(Some(e.into()));
                }

                Err(e) => {
                    queue_item.failure_count =
                        ActiveValue::Set(queue_item.failure_count.as_ref() + 1);
                    queue_item.scheduled_at = ActiveValue::Set(None);
                    queue_item.status = ActiveValue::Set(JobStatus::Failed);
                    queue_item.error_message = ActiveValue::Set(Some(e.into()));
                }
            }

            queue_item.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
            Some(queue_item.update(&tx).await?)
        } else {
            None
        };
        tx.commit().await?;
        Ok(model)
    }

    fn spawn_worker(self, join_set: &mut JoinSet<()>) {
        join_set.spawn(async move {
            loop {
                if self.cancel.is_cancelled() {
                    break;
                }

                match self.perform_one_queue_job().await {
                    Err(e) => {
                        tracing::error!("job error {e}");
                    }

                    Ok(Some(_)) => {}

                    Ok(None) => {
                        let sleep_duration =
                            Duration::from_millis(fastrand::u64(QUEUE_CHECK_INTERVAL));
                        tokio::select! {
                            () = self.cancel.cancelled() => break,
                            () = sleep(sleep_duration) => {}
                        }
                    }
                }
            }
        });
    }

    async fn supervise_workers(self) {
        self.schedule_recurring_tasks_if_needed().await.unwrap();
        let mut join_set = JoinSet::new();
        for _ in 0..QUEUE_WORKER_COUNT {
            self.clone().spawn_worker(&mut join_set);
        }

        while join_set.join_next().await.is_some() {
            if !self.cancel.is_cancelled() {
                tracing::error!("Worker task shut down. Restarting.");
                self.clone().spawn_worker(&mut join_set);
            }
        }
    }

    pub fn spawn_workers(self) -> JoinHandle<()> {
        tokio::task::spawn(self.supervise_workers())
    }
}
