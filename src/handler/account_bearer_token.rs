use crate::{
    entity::{Account, ApiToken, ApiTokens},
    Db,
};
use axum::extract::FromRef;
use axum::http::{header, request::Parts};
use sea_orm::ActiveModelTrait;
use trillium::{Conn, KnownHeaderName};
use trillium_api::FromConn;

#[derive(Clone, Debug)]
pub struct AccountBearerToken {
    pub account: Account,
    pub api_token: ApiToken,
}

#[trillium::async_trait]
impl FromConn for AccountBearerToken {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        if let Some(token) = conn.state::<AccountBearerToken>() {
            return Some(token.clone());
        }

        let db: &Db = conn.state()?;
        let token = conn
            .request_headers()
            .get_str(KnownHeaderName::Authorization)?
            .strip_prefix("Bearer ")?;
        let (api_token, account) = ApiTokens::load_and_check(token, db).await?;
        let api_token = api_token.mark_last_used().update(db).await.ok()?;
        let account_bearer_token = Self { account, api_token };
        conn.insert_state(account_bearer_token.clone());
        Some(account_bearer_token)
    }
}

// ---------------------------------------------------------------------------
// Axum extractor — mirrors the Trillium FromConn above
// ---------------------------------------------------------------------------

impl AccountBearerToken {
    /// Try to extract a bearer token from the request, returning `None` on
    /// any failure (missing header, invalid token, DB miss). This is
    /// intentionally fallible — [`PermissionsActor`](crate::PermissionsActor)
    /// tries this first and falls back to session-based user auth.
    pub(crate) async fn from_parts<S>(parts: &mut Parts, state: &S) -> Option<Self>
    where
        Db: FromRef<S>,
        S: Send + Sync,
    {
        // Cache: return early if already extracted for this request.
        if let Some(token) = parts.extensions.get::<Self>() {
            return Some(token.clone());
        }

        let bearer = parts
            .headers
            .get(header::AUTHORIZATION)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;

        let db = Db::from_ref(state);
        let (api_token, account) = ApiTokens::load_and_check(bearer, &db).await?;
        let api_token = api_token.mark_last_used().update(&db).await.ok()?;
        let result = Self { account, api_token };
        parts.extensions.insert(result.clone());
        Some(result)
    }
}
