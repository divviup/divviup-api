use crate::{
    entity::{Account, MembershipColumn, Memberships, NewTask, Task, Tasks, UpdateTask},
    handler::Error,
    user::User,
    DbConnExt,
};
use sea_orm::{prelude::*, ActiveModelTrait, ModelTrait};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;

pub async fn index(conn: &mut Conn, account: Account) -> Result<impl Handler, Error> {
    let db = conn.db();
    let tasks = account.find_related(Tasks).all(db).await?;
    if let Some(last_modified) = tasks.iter().map(|task| task.updated_at).max() {
        conn.set_last_modified(last_modified.into());
    }
    Ok(Json(tasks))
}

#[trillium::async_trait]
impl FromConn for Task {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let user = User::from_conn(conn).await?;
        let id = conn.param("task_id")?;

        let task = if user.is_admin() {
            Tasks::find_by_id(id).one(conn.db()).await
        } else {
            Tasks::find_by_id(id)
                .inner_join(Memberships)
                .filter(MembershipColumn::UserEmail.eq(&user.email))
                .one(conn.db())
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
    conn: &mut Conn,
    (account, Json(new_task)): (Account, Json<NewTask>),
) -> Result<impl Handler, Error> {
    let task = new_task.build(&account)?.insert(conn.db()).await?;
    Ok((Status::Created, Json(task)))
}

pub async fn show(conn: &mut Conn, task: Task) -> Json<Task> {
    conn.set_last_modified(task.updated_at.into());
    Json(task)
}

pub async fn update(
    conn: &mut Conn,
    (task, Json(update)): (Task, Json<UpdateTask>),
) -> Result<impl Handler, Error> {
    let task = update.build(task)?.update(conn.db()).await?;
    Ok(Json(task))
}
