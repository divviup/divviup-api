use crate::{
    entity::{
        Account, ApiToken, ApiTokenColumn, ApiTokens, MembershipColumn, Memberships, UpdateApiToken,
    },
    handler::Error,
    user::User,
    Db,
};
use sea_orm::{prelude::*, ActiveModelTrait, ModelTrait};
use trillium::{Conn, Handler, Status};
use trillium_api::{FromConn, Json};
use trillium_caching_headers::CachingHeadersExt;
use trillium_router::RouterConnExt;

pub async fn index(conn: &mut Conn, (account, db): (Account, Db)) -> Result<impl Handler, Error> {
    let api_tokens = account
        .find_related(ApiTokens)
        .filter(ApiTokenColumn::DeletedAt.is_null())
        .all(&db)
        .await?;
    if let Some(last_modified) = api_tokens
        .iter()
        .map(|api_token| api_token.updated_at)
        .max()
    {
        conn.set_last_modified(last_modified.into());
    }
    Ok(Json(api_tokens))
}

#[trillium::async_trait]
impl FromConn for ApiToken {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        let user = User::from_conn(conn).await?;
        let id = conn.param("api_token_id")?.parse::<Uuid>().ok()?;
        let db: &Db = conn.state()?;

        let api_token = if user.is_admin() {
            ApiTokens::find_by_id(id).one(db).await
        } else {
            ApiTokens::find_by_id(id)
                .inner_join(Memberships)
                .filter(MembershipColumn::UserEmail.eq(&user.email))
                .one(db)
                .await
        };

        match api_token {
            Ok(api_token) => api_token,
            Err(error) => {
                conn.set_state(Error::from(error));
                None
            }
        }
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
