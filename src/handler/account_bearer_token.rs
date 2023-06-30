use crate::{
    entity::{Account, Accounts, ApiToken, ApiTokenColumn, ApiTokens},
    Db,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use trillium::Conn;
use trillium_api::FromConn;
use trillium_http::KnownHeaderName;

#[derive(Clone, Debug)]
pub struct AccountBearerToken {
    pub account: Account,
    pub api_token: ApiToken,
}

const BEARER: &str = "Bearer ";

#[trillium::async_trait]
impl FromConn for AccountBearerToken {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        if let Some(token) = conn.state::<AccountBearerToken>() {
            return Some(token.clone());
        }

        let db: &Db = conn.state()?;
        let header = conn
            .request_headers()
            .get_str(KnownHeaderName::Authorization)?;

        if !header.starts_with(BEARER) {
            return None;
        }

        let header = &header[BEARER.len()..];
        let bytes = URL_SAFE_NO_PAD.decode(header).ok()?;
        let sha = Sha256::digest(bytes);
        let (api_token, account) = ApiTokens::find()
            .find_also_related(Accounts)
            .filter(ApiTokenColumn::TokenHash.eq(&*sha))
            .one(db)
            .await
            .ok()
            .flatten()?;

        let account = account?;
        let account_bearer_token = Self { account, api_token };

        conn.set_state(account_bearer_token.clone());
        Some(account_bearer_token)
    }
}
