use std::time::Duration;

use crate::{
    entity::{Account, MembershipColumn, Memberships, NewTask, Task, Tasks, UpdateTask},
    handler::Error,
    user::User,
    Db,
};
use sea_orm::{prelude::*, ActiveModelTrait, ModelTrait};
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
