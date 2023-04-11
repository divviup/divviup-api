use async_lock::RwLock;
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use trillium::KnownHeaderName;
use url::Url;

use crate::{client::expect_ok, ApiConfig, User};
type ClientConnector = trillium_rustls::RustlsConnector<trillium_tokio::TcpConnector>;
type Client = trillium_client::Client<ClientConnector>;

#[derive(Debug, Clone)]
pub struct Auth0Client {
    token: Arc<RwLock<Option<TokenWithExpiry>>>,
    client: Client,
    base_url: Url,
    secret: String,
    client_id: String,
}
impl Auth0Client {
    pub fn new(config: &ApiConfig) -> Self {
        Self {
            token: Arc::new(RwLock::new(None)),
            client: Client::new().with_default_pool(),
            base_url: config.auth_url.clone(),
            secret: config.auth_client_secret.clone(),
            client_id: config.auth_client_id.clone(),
        }
    }

    pub fn with_http_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    async fn get_new_token(&self) -> Result<String, crate::client::ClientError> {
        // we have to check again here because someone may have taken
        // a write lock and populated the token since we relinquished
        // our read lock
        let mut guard = self.token.write().await;
        if let Some(token) = &*guard {
            if token.is_fresh() {
                return Ok(token.token().to_string());
            }
        }

        guard.take();

        let mut conn = self
            .client
            .post(self.base_url.join("/oauth/token").unwrap())
            .with_header(KnownHeaderName::ContentType, "application/json")
            .with_json_body(&json!({
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.secret,
                "audience": self.base_url.join("/api/v2/").unwrap(),
            }))?
            .await?;

        expect_ok(&mut conn).await?;

        let token = conn.response_json::<Token>().await?;
        let token_string = token.access_token.clone();

        *guard = Some(token.into());
        Ok(token_string)
    }

    async fn token(&self) -> Result<String, crate::client::ClientError> {
        if let Some(token) = &*self.token.read().await {
            if token.is_fresh() {
                return Ok(token.token.clone());
            }
        }

        self.get_new_token().await
    }

    pub async fn create_user(&self, email: &str) -> Result<User, crate::client::ClientError> {
        let token = self.token().await?;
        let mut conn = self
            .client
            .post(self.base_url.join("/api/v2/users").unwrap())
            .with_header(KnownHeaderName::Accept, "application/json")
            .with_header(KnownHeaderName::ContentType, "application/json")
            .with_header(KnownHeaderName::Authorization, format!("Bearer {token}"))
            .with_json_body(&json!({
                "connection":"Username-Password-Authentication",
                "email": email,
                "password": std::iter::repeat_with(fastrand::alphanumeric).take(60).collect::<String>()
            }))?
            .await?;

        expect_ok(&mut conn).await?;

        Ok(conn.response_json().await?)
    }

    pub async fn users(&self) -> Result<Value, crate::client::ClientError> {
        let token = self.token().await?;
        let mut conn = self
            .client
            .get(self.base_url.join("/api/v2/users").unwrap())
            .with_header(KnownHeaderName::Accept, "application/json")
            .with_header(KnownHeaderName::Authorization, format!("Bearer {token}"))
            .await?;

        expect_ok(&mut conn).await?;

        let data = conn.response_json::<Value>().await?;
        Ok(data)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct Token {
    access_token: String,
    expires_in: u64,
    scope: String,
    token_type: String,
}

#[derive(Debug, Clone)]
struct TokenWithExpiry {
    token: String,
    expires_at: SystemTime,
}

impl TokenWithExpiry {
    pub fn is_fresh(&self) -> bool {
        SystemTime::now() < self.expires_at
    }

    fn token(&self) -> &str {
        self.token.as_ref()
    }
}

impl From<Token> for TokenWithExpiry {
    fn from(token: Token) -> Self {
        Self {
            token: token.access_token,
            expires_at: SystemTime::now() + Duration::from_secs(token.expires_in),
        }
    }
}
