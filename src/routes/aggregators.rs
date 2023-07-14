use crate::{
    entity::{Account, Aggregator, AggregatorColumn, Aggregators, NewAggregator, UpdateAggregator},
    handler::Error,
    Db, Permissions, PermissionsActor,
};
use sea_orm::{
    sea_query::{self, all, any},
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter,
};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_router::RouterConnExt;
use uuid::Uuid;

#[trillium::async_trait]
impl FromConn for Aggregator {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let db: &Db = conn.state()?;
        let id = conn.param("aggregator_id")?.parse::<Uuid>().ok()?;
        let aggregator = Aggregators::find_by_id(id)
            .filter(AggregatorColumn::DeletedAt.is_null())
            .one(db)
            .await;
        match aggregator {
            Ok(Some(aggregator)) => actor.if_allowed(conn.method(), aggregator),
            Ok(None) => None,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

impl Permissions for Aggregator {
    fn allow_read(&self, actor: &crate::PermissionsActor) -> bool {
        actor.is_admin()
            || match &self.account_id {
                Some(account_id) => actor.account_ids().contains(account_id),
                None => true,
            }
    }

    fn allow_write(&self, actor: &crate::PermissionsActor) -> bool {
        actor.is_admin()
            || match &self.account_id {
                Some(account_id) => actor.account_ids().contains(account_id),
                None => false,
            }
    }
}

pub async fn show(_: &mut Conn, aggregator: Aggregator) -> Json<Aggregator> {
    Json(aggregator)
}

pub async fn index(
    conn: &mut Conn,
    (db, account): (Db, Option<Account>),
) -> Result<Json<Vec<Aggregator>>, Error> {
    if conn.param("account_id").is_some() && account.is_none() {
        return Err(Error::NotFound);
    }

    Aggregators::find()
        .filter(all![
            match account {
                Some(account) => any![
                    AggregatorColumn::AccountId.eq(account.id),
                    AggregatorColumn::AccountId.is_null()
                ],
                None => any![AggregatorColumn::AccountId.is_null()],
            },
            AggregatorColumn::DeletedAt.is_null()
        ])
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn create(
    _: &mut Conn,
    (db, account, Json(new_aggregator)): (Db, Account, Json<NewAggregator>),
) -> Result<impl Handler, Error> {
    new_aggregator
        .build(Some(&account))?
        .insert(&db)
        .await
        .map_err(Error::from)
        .map(|agg| (Json(agg), Status::Created))
}

pub async fn delete(_: &mut Conn, (db, aggregator): (Db, Aggregator)) -> Result<Status, Error> {
    aggregator.tombstone().update(&db).await?;
    Ok(Status::NoContent)
}

pub async fn update(
    _: &mut Conn,
    (db, aggregator, Json(update_aggregator)): (Db, Aggregator, Json<UpdateAggregator>),
) -> Result<Json<Aggregator>, Error> {
    update_aggregator
        .build(aggregator)?
        .update(&db)
        .await
        .map_err(Error::from)
        .map(Json)
}

pub async fn admin_create(
    _: &mut Conn,
    (db, Json(new_aggregator)): (Db, Json<NewAggregator>),
) -> Result<impl Handler, Error> {
    new_aggregator
        .build(None)?
        .insert(&db)
        .await
        .map_err(Error::from)
        .map(|agg| (Json(agg), Status::Created))
}
