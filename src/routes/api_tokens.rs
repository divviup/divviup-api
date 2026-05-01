use crate::{
    entity::{Account, ApiToken, ApiTokenColumn, ApiTokens, UpdateApiToken},
    handler::{extract::extract_entity, extract::Json},
    Db, Error, Permissions, PermissionsActor,
};
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
use trillium::Conn;
use trillium_api::FromConn;
use trillium_router::RouterConnExt;
use uuid::Uuid;

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

impl<S> FromRequestParts<S> for ApiToken
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
        extract_entity::<ApiTokens, S>(parts, state, "api_token_id").await
    }
}

impl Permissions for ApiToken {
    fn allow_write(&self, actor: &PermissionsActor) -> bool {
        actor.is_admin() || actor.account_ids().contains(&self.account_id)
    }
}

pub async fn index(account: Account, State(db): State<Db>) -> Result<Json<Vec<ApiToken>>, Error> {
    account
        .find_related(ApiTokens)
        .filter(ApiTokenColumn::DeletedAt.is_null())
        .order_by_desc(ApiTokenColumn::CreatedAt)
        .all(&db)
        .await
        .map(Json)
        .map_err(Error::from)
}

pub async fn create(account: Account, State(db): State<Db>) -> Result<impl IntoResponse, Error> {
    let (api_token, token) = ApiToken::build(&account);
    let mut api_token = api_token.insert(&db).await?;
    api_token.token = Some(token);
    Ok((StatusCode::CREATED, Json(api_token)))
}

pub async fn delete(api_token: ApiToken, State(db): State<Db>) -> Result<StatusCode, Error> {
    api_token.tombstone().update(&db).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    api_token: ApiToken,
    State(db): State<Db>,
    Json(update): Json<UpdateApiToken>,
) -> Result<impl IntoResponse, Error> {
    let token = update.build(api_token)?.update(&db).await?;
    Ok((StatusCode::OK, Json(token)))
}
