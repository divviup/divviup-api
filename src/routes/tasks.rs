use crate::{
    entity::{Account, NewTask, Task, Tasks, UpdateTask},
    Db, Error, Permissions, PermissionsActor,
};
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait};
use std::time::Duration;
use time::OffsetDateTime;
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json, State};
use trillium_caching_headers::CachingHeadersExt;
use trillium_client::Client;
use trillium_router::RouterConnExt;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(Tasks)
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
    _: &mut Conn,
    (account, task, State(client), db): CreateArgs,
) -> Result<impl Handler, Error> {
    task.validate(account, &db)
        .await?
        .provision(client)
        .await?
        .insert(&db)
        .await
        .map_err(Into::into)
        .map(|task| (Status::Created, Json(task)))
}

async fn refresh_metrics_if_needed(task: Task, db: Db, client: Client) -> Result<Task, Error> {
    if OffsetDateTime::now_utc() - task.updated_at <= Duration::from_secs(5 * 60) {
        return Ok(task);
    }
    if let Some(aggregator) = task.first_party_aggregator(&db).await? {
        let metrics = aggregator.client(client).get_task_metrics(&task.id).await?;
        task.update_metrics(metrics, db).await.map_err(Into::into)
    } else {
        Ok(task)
    }
}

pub async fn show(
    conn: &mut Conn,
    (task, db, State(client)): (Task, Db, State<Client>),
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

pub mod collector_auth_tokens {
    use super::*;
    pub async fn index(
        _: &mut Conn,
        (task, db, State(client)): (Task, Db, State<Client>),
    ) -> Result<impl Handler, Error> {
        let leader = task.leader_aggregator(&db).await?;
        let client = leader.client(client);
        let task_response = client.get_task(&task.id).await?;
        Ok(Json([task_response.collector_auth_token]))
    }
}
