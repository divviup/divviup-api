use crate::{handler::Error, Db};
use sea_orm::ConnectionTrait;
use trillium::{Conn, Status};

pub async fn health_check(_: &mut Conn, db: Db) -> Result<Status, Error> {
    db.execute_unprepared("select 1").await?;
    Ok(Status::Ok)
}
