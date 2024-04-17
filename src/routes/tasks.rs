use crate::{
    clients::aggregator_client::TaskUploadMetrics,
    config::FeatureFlags,
    entity::{Account, NewTask, Task, TaskColumn, Tasks, UpdateTask},
    Crypter, Db, Error, Permissions, PermissionsActor,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use std::time::Duration;
use time::OffsetDateTime;
use tracing::warn;
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json, State};
use trillium_caching_headers::CachingHeadersExt;
use trillium_client::Client;
use trillium_router::RouterConnExt;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(Tasks)
        .filter(TaskColumn::DeletedAt.is_null())
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

impl Permissions for Task {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

#[trillium::async_trait]
impl FromConn for Task {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let db: &Db = conn.state()?;
        let id = conn.param("task_id")?;

        match Tasks::find_by_id(id).one(db).await {
            Ok(Some(task)) => actor.if_allowed(conn.method(), task),
            Ok(None) => None,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

type CreateArgs = (Account, Json<NewTask>, State<Client>, Db);
pub async fn create(
    conn: &mut Conn,
    (account, mut task, State(client), db): CreateArgs,
) -> Result<impl Handler, Error> {
    let crypter = conn.state().unwrap();
    task.normalize_and_validate(account, &db)
        .await?
        .provision(client, crypter)
        .await?
        .insert(&db)
        .await
        .map_err(Into::into)
        .map(|task| (Status::Created, Json(task)))
}

async fn refresh_metrics_if_needed(
    task: Task,
    db: Db,
    client: Client,
    crypter: &Crypter,
) -> Result<Task, Error> {
    if OffsetDateTime::now_utc() - task.updated_at <= Duration::from_secs(5 * 60) {
        return Ok(task);
    }
    let aggregator = task.leader_aggregator(&db).await?;
    let metrics = if aggregator.features.upload_metrics_enabled() {
        aggregator
            .client(client, crypter)?
            .get_task_upload_metrics(&task.id)
            .await?
    } else {
        TaskUploadMetrics::default()
    };
    task.update_task_upload_metrics(metrics, db)
        .await
        .map_err(Into::into)
}

pub async fn show(
    conn: &mut Conn,
    (task, db, State(client), State(feature_flags)): (Task, Db, State<Client>, State<FeatureFlags>),
) -> Result<Json<Task>, Error> {
    let task = if feature_flags.metrics_refresh_enabled {
        let crypter = conn.state().unwrap();
        refresh_metrics_if_needed(task, db, client, crypter).await?
    } else {
        task
    };
    conn.set_last_modified(task.updated_at.into());
    Ok(Json(task))
}

type UpdateArgs = (Task, Json<UpdateTask>, State<Client>, Db);
pub async fn update(
    conn: &mut Conn,
    (task, Json(update), client, db): UpdateArgs,
) -> Result<impl Handler, Error> {
    let crypter = conn.state().unwrap();
    update
        .update(&client, &db, crypter, task)
        .await?
        .update(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn delete(
    conn: &mut Conn,
    (task, client, db): (Task, State<Client>, Db),
) -> Result<impl Handler, Error> {
    if task.deleted_at.is_some() {
        return Ok(Status::NoContent);
    }

    let crypter = conn.state().unwrap();
    let now = OffsetDateTime::now_utc();

    let mut am = task.clone().into_active_model();
    // If the task has not already expired, mark the aggregator-side tasks as expired. This will
    // allow the aggregators to cease processing and eventually GC the task at their leisure.
    if task.expiration.is_none() || task.expiration > Some(now) {
        let update = UpdateTask::expiration(Some(now));

        // The leader aggregator drives DAP, so we _must_ succeed in setting its expiration.
        update
            .update_aggregator_expiration(
                task.leader_aggregator(&db).await?,
                &task.id,
                &client,
                crypter,
            )
            .await?;

        // Expiry helper-side is best-effort. It's plausible that the user is deleting tasks to a
        // BYOH that no longer exists, so we don't want to irritate them by forcing them to bring
        // the BYOH back up.
        //
        // If we fail to set expiry transiently, e.g. due to temporary server outage, it's no big
        // deal, because the leader is what drives the protocol forward. The helper shouldn't incur
        // load based on this task because we've ensured that the leader will stop.
        let _ = update
            .update_aggregator_expiration(
                task.leader_aggregator(&db).await?,
                &task.id,
                &client,
                crypter,
            )
            .await
            .map_err(|err| warn!(?err, "failed to expire helper-side task"));

        am.expiration = ActiveValue::Set(Some(now));
    }

    am.updated_at = ActiveValue::Set(now);
    am.deleted_at = ActiveValue::Set(Some(now));
    am.update(&db).await?;
    Ok(Status::NoContent)
}

pub mod collector_auth_tokens {
    use super::*;
    pub async fn index(
        conn: &mut Conn,
        (task, db, State(client)): (Task, Db, State<Client>),
    ) -> Result<impl Handler, Error> {
        let leader = task.leader_aggregator(&db).await?;
        if leader.features.token_hash_enabled() {
            Err(Error::NotFound)
        } else {
            let crypter = conn.state().unwrap();
            let client = leader.client(client, crypter)?;
            let task_response = client.get_task(&task.id).await?;
            Ok(Json([task_response.collector_auth_token]))
        }
    }
}
