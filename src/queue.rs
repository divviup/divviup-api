mod job;
pub use crate::entity::queue::{JobResult, JobStatus};
pub use job::*;

use crate::{
    entity::queue::{ActiveModel, Entity, Model},
    ApiConfig, Db,
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseBackend, DatabaseTransaction, DbErr, EntityTrait,
    IntoActiveModel, Set, Statement, TransactionTrait,
};
use std::{ops::Range, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{task::JoinSet, time::sleep};

/*
These configuration variables may eventually be useful to put on ApiConfig
*/
const MAX_RETRY: i32 = 5;
const QUEUE_CHECK_INTERVAL: Range<u64> = 10..20;
const QUEUE_WORKER_COUNT: u8 = 2;

fn schedule_based_on_failure_count(failure_count: i32) -> Option<OffsetDateTime> {
    if failure_count >= MAX_RETRY {
        None
    } else {
        Some(
            OffsetDateTime::now_utc()
                + Duration::from_millis(
                    4000_u64.pow(failure_count.try_into().unwrap()) + fastrand::u64(0..15000),
                ),
        )
    }
}

pub async fn dequeue_one(db: &Db, job_state: &SharedJobState) -> Result<Option<Model>, DbErr> {
    let tx = db.begin().await?;
    let model = if let Some(model) = next(&tx).await? {
        let mut active_model = model.into_active_model();
        let mut job = active_model
            .job
            .take()
            .expect("queue jobs should always have a Job");
        let result = job.perform(job_state, &tx).await;
        active_model.job = Set(job);
        active_model.updated_at = Set(OffsetDateTime::now_utc());
        match result {
            Ok(Some(job)) => {
                active_model.status = Set(JobStatus::Success);
                active_model.scheduled_at = Set(None);
                let mut next_job = ActiveModel::from(job);
                next_job.parent_id = Set(Some(*active_model.id.as_ref()));
                let next_job = next_job.insert(&tx).await?;
                active_model.result = Set(Some(JobResult::Child(next_job.id)));
            }

            Ok(None) => {
                active_model.scheduled_at = Set(None);
                active_model.status = Set(JobStatus::Success);
                active_model.result = Set(Some(JobResult::Complete));
            }

            Err(e) if e.is_retryable() => {
                active_model.failure_count = Set(active_model.failure_count.as_ref() + 1);
                let reschedule =
                    schedule_based_on_failure_count(*active_model.failure_count.as_ref());
                active_model.status =
                    Set(reschedule.map_or(JobStatus::Failed, |_| JobStatus::Pending));
                active_model.scheduled_at = Set(reschedule);
                active_model.result = Set(Some(JobResult::Error(e)));
            }

            Err(e) => {
                active_model.failure_count = Set(active_model.failure_count.as_ref() + 1);
                active_model.scheduled_at = Set(None);
                active_model.status = Set(JobStatus::Failed);
                active_model.result = Set(Some(JobResult::Error(e)));
            }
        }
        Some(active_model.update(&tx).await?)
    } else {
        None
    };
    tx.commit().await?;
    Ok(model)
}

fn spawn(join_set: &mut JoinSet<()>, db: &Db, job_state: &Arc<SharedJobState>) {
    let db = db.clone();
    let job_state = Arc::clone(job_state);
    join_set.spawn(async move {
        loop {
            match dequeue_one(&db, &job_state).await {
                Err(e) => {
                    tracing::error!("job error {e}");
                }

                Ok(Some(_)) => {}

                Ok(None) => {
                    sleep(Duration::from_secs(fastrand::u64(QUEUE_CHECK_INTERVAL))).await;
                }
            }
        }
    });
}

pub async fn run(db: Db, config: ApiConfig) {
    let mut join_set = JoinSet::new();
    let job_state = Arc::new(SharedJobState::from(&config));
    for _ in 0..QUEUE_WORKER_COUNT {
        spawn(&mut join_set, &db, &job_state);
    }

    while join_set.join_next().await.is_some() {
        tracing::error!("Worker task shut down. Restarting.");
        spawn(&mut join_set, &db, &job_state);
    }
}

async fn next(tx: &DatabaseTransaction) -> Result<Option<Model>, DbErr> {
    let select = match tx.get_database_backend() {
        backend @ DatabaseBackend::Postgres => Statement::from_sql_and_values(
            backend,
            r#"SELECT * FROM queue
               WHERE status = $1 AND (scheduled_at IS NULL OR scheduled_at < $2)
               ORDER BY updated_at ASC
               FOR UPDATE
               SKIP LOCKED
               LIMIT 1"#,
            [JobStatus::Pending.into(), OffsetDateTime::now_utc().into()],
        ),

        backend @ DatabaseBackend::Sqlite => Statement::from_sql_and_values(
            backend,
            r#"SELECT * FROM queue
               WHERE status = $1 AND (scheduled_at IS NULL OR scheduled_at < $2)
               ORDER BY updated_at ASC
               LIMIT 1"#,
            [JobStatus::Pending.into(), OffsetDateTime::now_utc().into()],
        ),

        _ => unimplemented!(),
    };

    Entity::find().from_raw_sql(select).one(tx).await
}
