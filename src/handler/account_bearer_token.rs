use crate::{
    entity::{Account, ApiToken, ApiTokens},
    Db,
};
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
        conn.set_state(account_bearer_token.clone());
        Some(account_bearer_token)
    }
}
