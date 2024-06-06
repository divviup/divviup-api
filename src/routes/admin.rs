use crate::{
    entity::queue::{self, Column, Entity, JobStatus, Model},
    handler::{admin_required, Error},
    Db,
};
use querystrong::QueryStrong;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryOrder, QuerySelect};
use trillium::{async_trait, Conn, Handler, Status};
use trillium_api::{api, FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::{router, RouterConnExt};
use uuid::Uuid;

pub fn routes() -> impl Handler {
    (
        api(admin_required),
        router()
            .get("/queue", api(index))
            .get("/queue/:job_id", api(show))
            .delete("/queue/:job_id", api(delete)),
    )
}

#[async_trait]
impl FromConn for queue::Model {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let db = Db::from_conn(conn).await?;
        let id: Uuid = conn.param("job_id")?.parse().ok()?;

        match Entity::find_by_id(id).one(&db).await {
            Ok(job) => job,
            Err(error) => {
                conn.insert_state(Error::from(error));
                None
            }
        }
    }
}

async fn index(conn: &mut Conn, db: Db) -> Result<Json<Vec<Model>>, Error> {
    let params = QueryStrong::parse(conn.querystring()).unwrap_or_default();
    let mut find = Entity::find();
    let query = QuerySelect::query(&mut find);
    match params.get_str("status") {
        Some("pending") => {
            query.cond_where(Column::Status.eq(JobStatus::Pending));
        }

        Some("success") => {
            query.cond_where(Column::Status.eq(JobStatus::Success));
        }

        Some("failed") => {
            query.cond_where(Column::Status.eq(JobStatus::Failed));
        }

        _ => {}
    }

    find.order_by_desc(Column::UpdatedAt)
        .limit(100)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

async fn show(conn: &mut Conn, queue_job: Model) -> Json<Model> {
    conn.set_last_modified(queue_job.updated_at.into());

    Json(queue_job)
}

async fn delete(_: &mut Conn, (queue_job, db): (Model, Db)) -> Result<Status, Error> {
    queue_job.delete(&db).await?;
    Ok(Status::NoContent)
}
