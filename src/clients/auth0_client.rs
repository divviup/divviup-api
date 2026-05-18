use async_lock::RwLock;
use axum::http::{header, StatusCode};
use educe::Educe;
use rand::distributions::{Alphanumeric, DistString};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use url::Url;

use crate::{
    clients::{ClientError, HttpClient, PostmarkClient, ResponseExt},
    Config,
};

#[derive(Clone, Educe)]
#[educe(Debug)]
pub struct Auth0Client {
    #[educe(Debug = false)]
    token: Arc<RwLock<Option<TokenWithExpiry>>>,
    client: HttpClient,
    #[educe(Debug = false)]
    secret: String,
    client_id: String,
    postmark_client: PostmarkClient,
}

fn generate_password() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 60)
}

fn extract_user_id(user: &serde_json::Value) -> Result<String, ClientError> {
    user.get("user_id")
        .ok_or_else(|| ClientError::Other("expected user_id".into()))?
        .as_str()
        .ok_or_else(|| ClientError::Other("expected user_id to be a string".into()))
        .map(String::from)
}

impl Auth0Client {
    pub fn new(config: &Config) -> Self {
        let client = config
            .client
            .clone()
            .with_base(config.auth_url.clone())
            .with_default_header(header::ACCEPT, "application/json");

        Self {
            token: Arc::new(RwLock::new(None)),
            client,
            secret: config.auth_client_secret.clone(),
            client_id: config.auth_client_id.clone(),
            postmark_client: PostmarkClient::new(config),
        }
    }

    pub async fn invite(
        &self,
        email: &str,
        account_name: &str,
    ) -> Result<(String, Url), ClientError> {
        let user_id = self.create_user(email).await?;
        let reset = self.password_reset(&user_id).await?;
        self.postmark_client
            .send_email_template(
                email,
                "user-invitation",
                &json!({
                    "email": email,
                    "account_name": account_name,
                    "action_url": reset
                }),
                None,
            )
            .await?;

        Ok((user_id.to_string(), reset))
    }

    pub async fn password_reset(&self, user_id: &str) -> Result<Url, ClientError> {
        self.post::<Value>(
            "/api/v2/tickets/password-change",
            &json!({ "user_id": user_id, "client_id": &self.client_id }),
        )
        .await?
        .get("ticket")
        .and_then(Value::as_str)
        .and_then(|u| Url::parse(u).ok())
        .ok_or(ClientError::Other("password reset".to_string()))
    }

    pub async fn create_user(&self, email: &str) -> Result<String, ClientError> {
        match self
            .post::<serde_json::Value>(
                "/api/v2/users",
                &json!({
                    "connection": "Username-Password-Authentication",
                    "email": email,
                    "password": generate_password(),
                    "verify_email": false
                }),
            )
            .await
        {
            Ok(user) => extract_user_id(&user),
            Err(ClientError::HttpStatusNotSuccess(e)) if e.status == Some(StatusCode::CONFLICT) => {
                self.find_user_id_by_email(email).await
            }
            Err(e) => Err(e),
        }
    }

    async fn find_user_id_by_email(&self, email: &str) -> Result<String, ClientError> {
        let query: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("q", &format!("email:\"{email}\""))
            .append_pair("search_engine", "v3")
            .finish();
        let users: Vec<serde_json::Value> = self.get(&format!("/api/v2/users?{query}")).await?;
        users
            .first()
            .ok_or_else(|| ClientError::Other("user not found after conflict".into()))
            .and_then(extract_user_id)
    }

    pub async fn users(&self) -> Result<Vec<Value>, ClientError> {
        self.get("/api/v2/users").await
    }

    // private below here

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

        let token: Token = self
            .client
            .post("/oauth/token")
            .json(&json!({
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.secret,
                "audience": self.client.build_url("/api/v2/").unwrap().to_string(),
            }))
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await?;

        *guard = Some(token.clone().into());
        Ok(token.access_token)
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
        self.client
            .post(path)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .json(json)
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    async fn get<T>(&self, path: &str) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let token = self.token().await?;
        self.client
            .get(path)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Educe)]
#[educe(Debug)]
pub struct Token {
    #[educe(Debug = false)]
    pub access_token: String,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
}

#[derive(Clone, Educe)]
#[educe(Debug)]
struct TokenWithExpiry {
    #[educe(Debug = false)]
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
