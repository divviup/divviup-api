use crate::{
    entity::{Account, ApiToken, ApiTokenColumn, ApiTokens, UpdateApiToken},
    Db, Error, Permissions, PermissionsActor,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_router::RouterConnExt;
use uuid::Uuid;

pub async fn index(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    account
        .find_related(ApiTokens)
        .filter(ApiTokenColumn::DeletedAt.is_null())
        .order_by_desc(ApiTokenColumn::CreatedAt)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

#[trillium::async_trait]
impl FromConn for ApiToken {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let actor = PermissionsActor::from_conn(conn).await?;
        let id = conn.param("api_token_id")?.parse::<Uuid>().ok()?;
        let db: &Db = conn.state()?;
        match ApiTokens::find_by_id(id).one(db).await {
            Ok(Some(api_token)) => actor.if_allowed(conn.method(), api_token),
            Ok(None) => None,
            Err(error) => {
                conn.insert_state(Error::from(error));
                None
            }
        }
    }
}

impl Permissions for ApiToken {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

pub async fn create(_: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    let (api_token, token) = ApiToken::build(&account);
    let mut api_token = api_token.insert(&db).await?;
    api_token.token = Some(token);
    Ok((Status::Created, Json(api_token)))
}

pub async fn delete(_: &mut Conn, (api_token, db): (ApiToken, Db)) -> Result<Status, Error> {
    api_token.tombstone().update(&db).await?;
    Ok(Status::NoContent)
}

pub async fn update(
    _: &mut Conn,
    (api_token, db, Json(update)): (ApiToken, Db, Json<UpdateApiToken>),
) -> Result<impl Handler, Error> {
    let token = update.build(api_token)?.update(&db).await?;
    Ok((Json(token), Status::Ok))
}
