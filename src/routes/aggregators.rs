use crate::{
    entity::{
        Account, Aggregator, AggregatorColumn, Aggregators, MembershipColumn, Memberships,
        NewAggregator, UpdateAggregator,
    },
    handler::Error,
    Db, User,
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
        let db: &Db = conn.state()?;
        let user: &User = conn.state()?;
        let id = conn
            .param("aggregator_id")
            .and_then(|s| Uuid::parse_str(s).ok())?;

        let aggregator = if user.is_admin() {
            Aggregators::find_by_id(id)
                .filter(AggregatorColumn::DeletedAt.is_null())
                .one(db)
                .await
        } else if conn.method().is_safe() {
            Aggregators::find_by_id(id)
                .left_join(Memberships)
                .filter(all![
                    any![
                        MembershipColumn::UserEmail.eq(&user.email),
                        AggregatorColumn::AccountId.is_null()
                    ],
                    AggregatorColumn::DeletedAt.is_null()
                ])
                .one(db)
                .await
        } else {
            Aggregators::find_by_id(id)
                .inner_join(Memberships)
                .filter(all![
                    MembershipColumn::UserEmail.eq(&user.email),
                    AggregatorColumn::DeletedAt.is_null()
                ])
                .one(db)
                .await
        };

        match aggregator {
            Ok(aggregator) => aggregator,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

pub async fn show(_: &mut Conn, aggregator: Aggregator) -> Json<Aggregator> {
    Json(aggregator)
}

pub async fn index(
    _: &mut Conn,
    (db, account): (Db, Account),
) -> Result<Json<Vec<Aggregator>>, Error> {
    Aggregators::find()
        .filter(all![
            any![
                AggregatorColumn::AccountId.eq(account.id),
                AggregatorColumn::AccountId.is_null()
            ],
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
) -> Result<Json<Aggregator>, Error> {
    new_aggregator
        .build(None)?
        .insert(&db)
        .await
        .map_err(Error::from)
        .map(Json)
}
