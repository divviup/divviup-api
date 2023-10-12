use crate::{
    entity::{
        Account, CollectorCredential, CollectorCredentialColumn, CollectorCredentials,
        NewCollectorCredential, UpdateCollectorCredential,
    },
    Db, Error, Permissions, PermissionsActor,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;
use uuid::Uuid;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(CollectorCredentials)
        .filter(CollectorCredentialColumn::DeletedAt.is_null())
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

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
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

impl Permissions for CollectorCredential {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

pub async fn show(
    conn: &mut Conn,
    collector_credential: CollectorCredential,
) -> Result<Json<CollectorCredential>, Error> {
    conn.set_last_modified(collector_credential.updated_at.into());
    Ok(Json(collector_credential))
}

pub async fn create(
    _: &mut Conn,
    (account, db, Json(collector_credential)): (Account, Db, Json<NewCollectorCredential>),
) -> Result<impl Handler, Error> {
    let (collector_credential, token) = collector_credential.build(&account)?;
    let mut collector_credential = collector_credential.insert(&db).await?;
    collector_credential.token = Some(token);
    Ok((Status::Created, Json(collector_credential)))
}

pub async fn delete(
    _: &mut Conn,
    (collector_credential, db): (CollectorCredential, Db),
) -> Result<Status, Error> {
    collector_credential.tombstone().update(&db).await?;
    Ok(Status::NoContent)
}

pub async fn update(
    _: &mut Conn,
    (collector_credential, db, Json(update)): (
        CollectorCredential,
        Db,
        Json<UpdateCollectorCredential>,
    ),
) -> Result<impl Handler, Error> {
    let token = update.build(collector_credential)?.update(&db).await?;
    Ok((Json(token), Status::Ok))
}
