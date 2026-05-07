use crate::{
    config::FeatureFlags,
    entity::{Account, Aggregator, AggregatorColumn, Aggregators, NewAggregator, UpdateAggregator},
    handler::extract::{extract_entity, Json},
    Crypter, Db, Error, Permissions, PermissionsActor,
};
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::{request::Parts, StatusCode};
use axum::response::IntoResponse;
use sea_orm::{
    sea_query::{all, any},
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter,
};
use trillium::Conn;
use trillium_api::FromConn;
use trillium_client::Client;
use trillium_router::RouterConnExt;
use uuid::Uuid;

#[trillium::async_trait]
impl FromConn for Aggregator {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let db: &Db = conn.state()?;
        let id = conn.param("aggregator_id")?.parse::<Uuid>().ok()?;
        let aggregator = Aggregators::find_by_id(id).one(db).await;
        match aggregator {
            Ok(Some(aggregator)) => actor.if_allowed(conn.method(), aggregator),
            Ok(None) => None,
            Err(error) => {
                conn.insert_state(Error::from(error));
                None
            }
        }
    }
}

impl<S> FromRequestParts<S> for Aggregator
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        extract_entity::<Aggregators, S>(parts, state, "aggregator_id").await
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

pub mod axum_handler {
    use super::*;

    pub async fn show(aggregator: Aggregator) -> Json<Aggregator> {
        Json(aggregator)
    }

    pub async fn index_shared(
        _actor: PermissionsActor,
        State(db): State<Db>,
    ) -> Result<Json<Vec<Aggregator>>, Error> {
        Ok(Json(
            Aggregators::find()
                .filter(all![
                    AggregatorColumn::AccountId.is_null(),
                    AggregatorColumn::DeletedAt.is_null()
                ])
                .all(&db)
                .await?,
        ))
    }

    pub async fn index_for_account(
        account: Account,
        State(db): State<Db>,
    ) -> Result<Json<Vec<Aggregator>>, Error> {
        Ok(Json(
            Aggregators::find()
                .filter(all![
                    any![
                        AggregatorColumn::AccountId.eq(account.id),
                        AggregatorColumn::AccountId.is_null()
                    ],
                    AggregatorColumn::DeletedAt.is_null()
                ])
                .all(&db)
                .await?,
        ))
    }

    pub async fn create(
        account: Account,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        State(feature_flags): State<FeatureFlags>,
        Json(new_aggregator): Json<NewAggregator>,
    ) -> Result<impl IntoResponse, Error> {
        let aggregator = new_aggregator
            .build(
                Some(&account),
                client,
                &crypter,
                feature_flags.ssrf_validation_enabled,
            )
            .await?
            .insert(&db)
            .await?;
        Ok((StatusCode::CREATED, Json(aggregator)))
    }

    pub async fn update(
        aggregator: Aggregator,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        Json(update_aggregator): Json<UpdateAggregator>,
    ) -> Result<Json<Aggregator>, Error> {
        Ok(Json(
            update_aggregator
                .build(aggregator, client, &crypter)
                .await?
                .update(&db)
                .await?,
        ))
    }

    pub async fn delete(aggregator: Aggregator, State(db): State<Db>) -> Result<StatusCode, Error> {
        aggregator.tombstone().update(&db).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    pub async fn admin_create(
        actor: PermissionsActor,
        State(db): State<Db>,
        State(client): State<Client>,
        State(crypter): State<Crypter>,
        State(feature_flags): State<FeatureFlags>,
        Json(new_aggregator): Json<NewAggregator>,
    ) -> Result<impl IntoResponse, Error> {
        if !actor.is_admin() {
            return Err(Error::NotFound);
        }

        let aggregator = new_aggregator
            .build(
                None,
                client,
                &crypter,
                feature_flags.ssrf_validation_enabled,
            )
            .await?
            .insert(&db)
            .await?;
        Ok((StatusCode::CREATED, Json(aggregator)))
    }
}
