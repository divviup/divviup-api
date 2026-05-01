use crate::{
    entity::{
        Account, CollectorCredential, CollectorCredentialColumn, CollectorCredentials,
        NewCollectorCredential, UpdateCollectorCredential,
    },
    handler::{extract::extract_entity, extract::Json},
    Db, Error, Permissions, PermissionsActor,
};
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use trillium::Conn;
use trillium_api::FromConn;
use trillium_router::RouterConnExt;
use uuid::Uuid;

#[trillium::async_trait]
impl FromConn for CollectorCredential {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let id = conn
            .param("collector_credential_id")?
            .parse::<Uuid>()
            .ok()?;
        let db: &Db = conn.state()?;
        match CollectorCredentials::find_by_id(id).one(db).await {
            Ok(Some(collector_credential)) => actor.if_allowed(conn.method(), collector_credential),
            Ok(None) => None,
            Err(error) => {
                conn.insert_state(Error::from(error));
                None
            }
        }
    }
}

impl<S> FromRequestParts<S> for CollectorCredential
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        extract_entity::<CollectorCredentials, S>(parts, state, "collector_credential_id").await
    }
}

impl Permissions for CollectorCredential {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

pub async fn index(
    account: Account,
    State(db): State<Db>,
) -> Result<Json<Vec<CollectorCredential>>, Error> {
    account
        .find_related(CollectorCredentials)
        .filter(CollectorCredentialColumn::DeletedAt.is_null())
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn show(collector_credential: CollectorCredential) -> impl IntoResponse {
    let last_modified = httpdate::fmt_http_date(collector_credential.updated_at.into());
    (
        [(header::LAST_MODIFIED, last_modified)],
        Json(collector_credential),
    )
}

pub async fn create(
    account: Account,
    State(db): State<Db>,
    Json(collector_credential): Json<NewCollectorCredential>,
) -> Result<impl IntoResponse, Error> {
    let (collector_credential, token) = collector_credential.build(&account)?;
    let mut collector_credential = collector_credential.insert(&db).await?;
    collector_credential.token = Some(token);
    Ok((StatusCode::CREATED, Json(collector_credential)))
}

pub async fn delete(
    collector_credential: CollectorCredential,
    State(db): State<Db>,
) -> Result<StatusCode, Error> {
    collector_credential.tombstone().update(&db).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    collector_credential: CollectorCredential,
    State(db): State<Db>,
    Json(update): Json<UpdateCollectorCredential>,
) -> Result<impl IntoResponse, Error> {
    let token = update.build(collector_credential)?.update(&db).await?;
    Ok((StatusCode::OK, Json(token)))
}
