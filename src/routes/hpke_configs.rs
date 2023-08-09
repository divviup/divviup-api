use crate::{
    entity::{Account, HpkeConfig, HpkeConfigColumn, HpkeConfigs, NewHpkeConfig, UpdateHpkeConfig},
    Db, Error, Permissions, PermissionsActor,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_router::RouterConnExt;
use uuid::Uuid;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(HpkeConfigs)
        .filter(HpkeConfigColumn::DeletedAt.is_null())
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

#[trillium::async_trait]
impl FromConn for HpkeConfig {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let id = conn.param("hpke_config_id")?.parse::<Uuid>().ok()?;
        let db: &Db = conn.state()?;
        match HpkeConfigs::find_by_id(id).one(db).await {
            Ok(Some(hpke_config)) => actor.if_allowed(conn.method(), hpke_config),
            Ok(None) => None,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
    }
}

impl Permissions for HpkeConfig {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

pub async fn create(
    _: &mut Conn,
    (account, db, Json(hpke_config)): (Account, Db, Json<NewHpkeConfig>),
) -> Result<impl Handler, Error> {
    let hpke_config = hpke_config.build(&account)?.insert(&db).await?;
    Ok((Status::Created, Json(hpke_config)))
}

pub async fn delete(_: &mut Conn, (hpke_config, db): (HpkeConfig, Db)) -> Result<Status, Error> {
    hpke_config.tombstone().update(&db).await?;
    Ok(Status::NoContent)
}

pub async fn update(
    _: &mut Conn,
    (hpke_config, db, Json(update)): (HpkeConfig, Db, Json<UpdateHpkeConfig>),
) -> Result<impl Handler, Error> {
    let token = update.build(hpke_config)?.update(&db).await?;
    Ok((Json(token), Status::Ok))
}
