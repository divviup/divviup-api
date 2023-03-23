use crate::{handler::Error, AggregatorClient, Db};
use sqlx::Executor;
use trillium::{Conn, Status};

pub async fn health_check(
    _: &mut Conn,
    (db, api_client): (Db, AggregatorClient),
) -> Result<Status, Error> {
    db.get_postgres_connection_pool()
        .execute("select 1")
        .await?;

    api_client.health_check().await?;
    Ok(Status::Ok)
}
