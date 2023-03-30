use crate::{handler::Error, AggregatorClient, Db};
use sea_orm::ConnectionTrait;
use trillium::{Conn, Status};

pub async fn health_check(
    _: &mut Conn,
    (db, api_client): (Db, AggregatorClient),
) -> Result<Status, Error> {
    db.execute_unprepared("select 1").await?;
    api_client.health_check().await?;
    Ok(Status::Ok)
}
