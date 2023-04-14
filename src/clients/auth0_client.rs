use async_lock::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::task::spawn;
use trillium::{
    Conn,
    KnownHeaderName::{Accept, Authorization, ContentType},
};
use trillium_api::FromConn;
use url::Url;

use crate::{
    clients::{expect_ok, Client, ClientError, PostmarkClient},
    entity::{Account, Membership},
    ApiConfig,
};

#[derive(Debug, Clone)]
pub struct Auth0Client {
    token: Arc<RwLock<Option<TokenWithExpiry>>>,
    client: Client,
    base_url: Url,
    secret: String,
    client_id: String,
    postmark_client: PostmarkClient,
}

#[trillium::async_trait]
impl FromConn for Auth0Client {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

impl Auth0Client {
    pub fn new(config: &ApiConfig) -> Self {
        Self {
            token: Arc::new(RwLock::new(None)),
            client: Client::new().with_default_pool(),
            base_url: config.auth_url.clone(),
            secret: config.auth_client_secret.clone(),
            client_id: config.auth_client_id.clone(),
            postmark_client: PostmarkClient::new(config),
        }
    }

    pub fn with_http_client(mut self, client: Client) -> Self {
        self.client = client.clone();
        self.postmark_client.set_http_client(client);
        self
    }

    async fn get_new_token(&self) -> Result<String, ClientError> {
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
            .with_header(ContentType, "application/json")
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

    async fn token(&self) -> Result<String, ClientError> {
        if let Some(token) = &*self.token.read().await {
            if token.is_fresh() {
                return Ok(token.token.clone());
            }
        }

        self.get_new_token().await
    }

    async fn post<T>(&self, path: &str, json: &impl Serialize) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let token = self.token().await?;
        let mut conn = self
            .client
            .post(self.base_url.join(path).unwrap())
            .with_header(Accept, "application/json")
            .with_header(ContentType, "application/json")
            .with_header(Authorization, format!("Bearer {token}"))
            .with_json_body(json)?
            .await?;
        expect_ok(&mut conn).await?;
        Ok(conn.response_json().await?)
    }

    async fn get<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let token = self.token().await?;
        let mut conn = self
            .client
            .get(self.base_url.join(path).unwrap())
            .with_header(Accept, "application/json")
            .with_header(Authorization, format!("Bearer {token}"))
            .await?;
        expect_ok(&mut conn).await?;
        Ok(conn.response_json().await?)
    }

    pub async fn invite(&self, email: &str, account_name: &str) -> Result<(), ClientError> {
        let user = self.create_user(email).await?;
        let user_id = user.get("user_id").unwrap().as_str().unwrap();
        let reset = self.password_reset(user_id).await?;

        self.postmark_client
            .send_email_template(
                email,
                "user-invitation",
                &json!({
                    "email": email,
                    "account_name": account_name,
                    "action_url": reset
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn password_reset(&self, user_id: &str) -> Result<Url, ClientError> {
        let value: Value = self
            .post(
                "/api/v2/tickets/password-change",
                &json!({ "user_id": user_id, "client_id": &self.client_id }),
            )
            .await?;
        value
            .get("ticket")
            .and_then(Value::as_str)
            .and_then(|u| Url::parse(u).ok())
            .ok_or(ClientError::Other(format!("password reset")))
    }

    pub async fn create_user(&self, email: &str) -> Result<Value, ClientError> {
        self.post("/api/v2/users", &json!({
            "connection": "Username-Password-Authentication",
            "email": email,
            "password": std::iter::repeat_with(fastrand::alphanumeric).take(60).collect::<String>(),
            "verify_email": false
        })).await
    }

    pub async fn users(&self) -> Result<Vec<Value>, ClientError> {
        self.get("/api/v2/users").await
    }

    pub(crate) fn spawn_invitation_task(&self, membership: Membership, account: Account) {
        let client = self.clone();
        spawn(async move {
            match client.invite(&membership.user_email, &account.name).await {
                Ok(()) => log::info!("sent email regarding {membership:?}"),

                Err(e) => log::error!(
                    "error while sending email regarding membership {membership:?}: \n\n{e}"
                ),
            }
        });
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
