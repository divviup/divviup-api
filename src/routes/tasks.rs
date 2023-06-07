use std::time::Duration;

use crate::{
    clients::{aggregator_client::TaskCreate, AggregatorClient},
    entity::{
        task::build_task, Account, MembershipColumn, Memberships, NewTask, Task, Tasks, UpdateTask,
    },
    handler::Error,
    user::User,
    Db,
};
use sea_orm::{prelude::*, ActiveModelTrait, ModelTrait};
use time::OffsetDateTime;
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;

pub async fn index(conn: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    let tasks = account.find_related(Tasks).all(&db).await?;
    if let Some(last_modified) = tasks.iter().map(|task| task.updated_at).max() {
        conn.set_last_modified(last_modified.into());
    }
    Ok(Json(tasks))
}

#[trillium::async_trait]
impl FromConn for Task {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let db = Db::from_conn(conn).await?;
        let user = User::from_conn(conn).await?;
        let id = conn.param("task_id")?;

        let task = if user.is_admin() {
            Tasks::find_by_id(id).one(&db).await
        } else {
            Tasks::find_by_id(id)
                .inner_join(Memberships)
                .filter(MembershipColumn::UserEmail.eq(&user.email))
                .one(&db)
                .await
        };

        match task {
            Ok(task) => task,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

type CreateArgs = (Account, Json<NewTask>, AggregatorClient, Db);
pub async fn create(
    conn: &mut Conn,
    (account, Json(task), api_client, db): CreateArgs,
) -> Result<impl Handler, Error> {
    task.validate()?;
    let config = conn.state().ok_or(Error::NotFound)?;
    let task_create = TaskCreate::build(task.clone(), config)?;
    let api_response = api_client.create_task(task_create).await?;
    let task = build_task(task, api_response, &account).insert(&db).await?;
    Ok((Status::Created, Json(task)))
}

async fn refresh_metrics_if_needed(
    task: Task,
    db: Db,
    client: AggregatorClient,
) -> Result<Task, Error> {
    if OffsetDateTime::now_utc() - task.updated_at > Duration::from_secs(5 * 60) {
        let metrics = client.get_task_metrics(&task.id).await?;
        task.update_metrics(metrics, db).await.map_err(Into::into)
    } else {
        Ok(task)
    }
}

pub async fn show(
    conn: &mut Conn,
    (task, db, client): (Task, Db, AggregatorClient),
) -> Result<Json<Task>, Error> {
    let task = refresh_metrics_if_needed(task, db, client).await?;
    conn.set_last_modified(task.updated_at.into());
    Ok(Json(task))
}

pub async fn update(
    _: &mut Conn,
    (task, Json(update), db): (Task, Json<UpdateTask>, Db),
) -> Result<impl Handler, Error> {
    let task = update.build(task)?.update(&db).await?;
    Ok(Json(task))
}
