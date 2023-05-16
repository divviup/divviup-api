mod job;
pub use crate::entity::queue::JobStatus;
pub use job::*;

use crate::{
    entity::queue::{ActiveModel, Entity, Model},
    ApiConfig, Db, DivviupApi,
};
use sea_orm::{ActiveModelTrait, DbErr, IntoActiveModel, Set, TransactionTrait};
use std::{ops::Range, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{
    task::{JoinHandle, JoinSet},
    time::sleep,
};
use trillium_tokio::{CloneCounterObserver, Stopper};

#[derive(Clone, Debug)]
pub struct Queue {
    observer: CloneCounterObserver,
    stopper: Stopper,
    db: Db,
    job_state: Arc<SharedJobState>,
}
/*
These configuration variables may eventually be useful to put on ApiConfig
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

impl From<&DivviupApi> for Queue {
    fn from(app: &DivviupApi) -> Self {
        Self::new(app.db(), app.config())
    }
}

impl Queue {
    pub fn new(db: &Db, config: &ApiConfig) -> Self {
        Self {
            observer: Default::default(),
            db: db.clone(),
            stopper: Default::default(),
            job_state: Arc::new(config.into()),
        }
    }

    pub fn with_observer(mut self, observer: CloneCounterObserver) -> Self {
        self.observer = observer;
        self
    }

    pub fn with_stopper(mut self, stopper: Stopper) -> Self {
        self.stopper = stopper;
        self
    }

    pub async fn perform_one_queue_job(&self) -> Result<Option<Model>, DbErr> {
        let tx = self.db.begin().await?;
        let model = if let Some(queue_item) = Entity::next(&tx).await? {
            let _counter = self.observer.counter();
            let mut queue_item = queue_item.into_active_model();

            let mut job = queue_item.job.take().ok_or_else(|| {
                DbErr::Custom(String::from(
                    r#"Queue item found without a job.
                       We believe this to be unreachable"#,
                ))
            })?;

            let result = job.perform(&self.job_state, &tx).await;
            queue_item.job = Set(job);

            match result {
                Ok(Some(next_job)) => {
                    queue_item.status = Set(JobStatus::Success);
                    queue_item.scheduled_at = Set(None);

                    let mut next_job = ActiveModel::from(next_job);
                    next_job.parent_id = Set(Some(*queue_item.id.as_ref()));
                    let next_job = next_job.insert(&tx).await?;
                    queue_item.child_id = Set(Some(next_job.id));
                }

                Ok(None) => {
                    queue_item.scheduled_at = Set(None);
                    queue_item.status = Set(JobStatus::Success);
                }

                Err(e) if e.is_retryable() => {
                    queue_item.failure_count = Set(queue_item.failure_count.as_ref() + 1);
                    let reschedule =
                        reschedule_based_on_failure_count(*queue_item.failure_count.as_ref());
                    queue_item.status =
                        Set(reschedule.map_or(JobStatus::Failed, |_| JobStatus::Pending));
                    queue_item.scheduled_at = Set(reschedule);
                    queue_item.error_message = Set(Some(e));
                }

                Err(e) => {
                    queue_item.failure_count = Set(queue_item.failure_count.as_ref() + 1);
                    queue_item.scheduled_at = Set(None);
                    queue_item.status = Set(JobStatus::Failed);
                    queue_item.error_message = Set(Some(e));
                }
            }

            queue_item.updated_at = Set(OffsetDateTime::now_utc());
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
                if self.stopper.is_stopped() {
                    break;
                }

                match self.perform_one_queue_job().await {
                    Err(e) => {
                        tracing::error!("job error {e}");
                    }

                    Ok(Some(_)) => {}

                    Ok(None) => {
                        let sleep_future =
                            sleep(Duration::from_millis(fastrand::u64(QUEUE_CHECK_INTERVAL)));
                        self.stopper.stop_future(sleep_future).await;
                    }
                }
            }
        });
    }

    async fn supervise_workers(self) {
        let mut join_set = JoinSet::new();
        for _ in 0..QUEUE_WORKER_COUNT {
            self.clone().spawn_worker(&mut join_set);
        }

        while join_set.join_next().await.is_some() {
            if !self.stopper.is_stopped() {
                tracing::error!("Worker task shut down. Restarting.");
                self.clone().spawn_worker(&mut join_set);
            }
        }
    }

    pub fn spawn_workers(self) -> JoinHandle<()> {
        tokio::task::spawn(self.supervise_workers())
    }
}
