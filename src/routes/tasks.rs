use crate::{
    clients::aggregator_client::{api_types::TaskAggregationJobMetrics, TaskUploadMetrics},
    config::FeatureFlags,
    entity::{Account, NewTask, Task, TaskColumn, Tasks, UpdateTask},
    handler::extract::Json,
    Crypter, Db, Error, Permissions, PermissionsActor,
};
use axum::extract::{FromRef, FromRequestParts, Path, Query, State};
use axum::http::{header, request::Parts, StatusCode};
use axum::response::IntoResponse;
use httpdate::fmt_http_date;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::join;
use tracing::warn;
use trillium::Conn;
use trillium_api::FromConn;
use trillium_client::Client;
use trillium_router::RouterConnExt;

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
                conn.insert_state(Error::from(error));
                None
            }
        }
    }
}

impl<S> FromRequestParts<S> for Task
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::NotFound)?;

        let id = params.get("task_id").ok_or(Error::NotFound)?;

        let actor = PermissionsActor::from_request_parts(parts, state).await?;
        let db = Db::from_ref(state);

        let task = Tasks::find_by_id(id)
            .one(&db)
            .await?
            .ok_or(Error::AccessDenied)?;

        actor
            .if_allowed_http(&parts.method, task)
            .ok_or(Error::AccessDenied)
    }
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
            .client(client.clone(), crypter)?
            .get_task_upload_metrics(&task.id)
            .await?
    } else {
        TaskUploadMetrics::default()
    };
    let task = task.update_task_upload_metrics(metrics, db.clone()).await?;

    let metrics = if aggregator.features.aggregation_job_metrics_enabled() {
        aggregator
            .client(client, crypter)?
            .get_task_aggregation_job_metrics(&task.id)
            .await?
    } else {
        TaskAggregationJobMetrics::default()
    };
    task.update_task_aggregation_job_metrics(metrics, db)
        .await
        .map_err(Into::into)
}

pub mod axum_handler {
    use super::*;

    pub async fn index(account: Account, State(db): State<Db>) -> Result<Json<Vec<Task>>, Error> {
        Ok(Json(
            account
                .find_related(Tasks)
                .filter(TaskColumn::DeletedAt.is_null())
                .all(&db)
                .await?,
        ))
    }

    pub async fn create(
        account: Account,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        Json(mut new_task): Json<NewTask>,
    ) -> Result<impl IntoResponse, Error> {
        let task = new_task
            .normalize_and_validate(account, &db)
            .await?
            .provision(client, &crypter)
            .await?
            .insert(&db)
            .await?;
        Ok((StatusCode::CREATED, Json(task)))
    }

    pub async fn show(
        task: Task,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        State(feature_flags): State<FeatureFlags>,
    ) -> Result<impl IntoResponse, Error> {
        let task = if feature_flags.metrics_refresh_enabled {
            refresh_metrics_if_needed(task, db, client, &crypter).await?
        } else {
            task
        };
        let last_modified = fmt_http_date(task.updated_at.into());
        Ok(([(header::LAST_MODIFIED, last_modified)], Json(task)))
    }

    pub async fn update(
        task: Task,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        Json(update): Json<UpdateTask>,
    ) -> Result<Json<Task>, Error> {
        Ok(Json(
            update
                .update(&client, &db, &crypter, task)
                .await?
                .update(&db)
                .await?,
        ))
    }

    #[derive(Deserialize)]
    pub struct DeleteParams {
        #[serde(default)]
        force: bool,
    }

    pub async fn delete(
        task: Task,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        Query(params): Query<DeleteParams>,
    ) -> Result<StatusCode, Error> {
        if task.deleted_at.is_some() {
            return Ok(StatusCode::NO_CONTENT);
        }

        let now = OffsetDateTime::now_utc();

        let mut am = task.clone().into_active_model();
        if task.expiration.is_none() || task.expiration > Some(now) {
            let update = UpdateTask::expiration(Some(now));

            let (leader_result, helper_result) = join!(
                update.update_aggregator_expiration(
                    task.leader_aggregator(&db).await?,
                    &task.id,
                    &client,
                    &crypter,
                ),
                update.update_aggregator_expiration(
                    task.helper_aggregator(&db).await?,
                    &task.id,
                    &client,
                    &crypter,
                )
            );

            if params.force {
                let _ = leader_result
                    .map_err(|err| warn!(?err, "failed to expire leader-side task, ignoring"));
                let _ = helper_result
                    .map_err(|err| warn!(?err, "failed to expire helper-side task, ignoring"));
            } else {
                leader_result?;
                helper_result?;
            }

            am.expiration = ActiveValue::Set(Some(now));
        }

        am.updated_at = ActiveValue::Set(now);
        am.deleted_at = ActiveValue::Set(Some(now));
        am.update(&db).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
