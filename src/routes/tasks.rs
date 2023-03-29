use crate::{
    entity::{
        task::build_task, Account, MembershipColumn, Memberships, NewTask, Task, Tasks, UpdateTask,
    },
    handler::Error,
    user::User,
    AggregatorClient, Db,
};
use sea_orm::{prelude::*, ActiveModelTrait, ModelTrait};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;
use validator::Validate;

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

pub async fn create(
    _: &mut Conn,
    (account, Json(task), api_client, db): (Account, Json<NewTask>, AggregatorClient, Db),
) -> Result<impl Handler, Error> {
    task.validate()?;
    let api_response = api_client.create_task(task.clone()).await?;
    let task = build_task(task, api_response, &account).insert(&db).await?;
    Ok((Status::Created, Json(task)))
}

pub async fn show(conn: &mut Conn, task: Task) -> Json<Task> {
    conn.set_last_modified(task.updated_at.into());
    Json(task)
}

pub async fn update(
    _: &mut Conn,
    (task, Json(update), db): (Task, Json<UpdateTask>, Db),
) -> Result<impl Handler, Error> {
    let task = update.build(task)?.update(&db).await?;
    Ok(Json(task))
}
