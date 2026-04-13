#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

mod account;
mod aggregator;
mod api_token;
mod collector_credentials;
pub mod dp_strategy;
mod membership;
mod protocol;
mod task;
mod validation_errors;

pub const CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";
pub const DEFAULT_URL: &str = "https://api.divviup.org/";
pub const USER_AGENT: &str = concat!("divviup-client/", env!("CARGO_PKG_VERSION"));

use base64::{engine::general_purpose::STANDARD, Engine};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE as CONTENT_TYPE_HEADER},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::fmt::Display;
use time::format_description::well_known::Rfc3339;

pub use account::Account;
pub use aggregator::{Aggregator, CollectorAuthenticationToken, NewAggregator, Role};
pub use api_token::ApiToken;
pub use collector_credentials::CollectorCredential;
pub use http;
pub use janus_messages::{
    codec::{CodecError, Decode, Encode},
    HpkeConfig, HpkePublicKey,
};
pub use membership::Membership;
pub use num_bigint::BigUint;
pub use num_rational::Ratio;
pub use protocol::Protocol;
pub use reqwest;
pub use task::{Histogram, NewTask, SumVec, Task, Vdaf};
pub use time::OffsetDateTime;
pub use url::Url;
pub use uuid::Uuid;
pub use validation_errors::ValidationErrors;

#[cfg(feature = "admin")]
pub use aggregator::NewSharedAggregator;

#[derive(Debug, Clone)]
pub struct DivviupClient {
    client: reqwest::Client,
    base_url: Url,
    token: String,
}

impl DivviupClient {
    pub fn new(token: impl Display, client: reqwest::Client) -> Self {
        Self {
            client,
            // Safety: DEFAULT_URL is a compile-time constant known to be valid.
            base_url: Url::parse(DEFAULT_URL).unwrap(),
            token: token.to_string(),
        }
    }

    pub fn with_url(mut self, url: Url) -> Self {
        self.set_url(url);
        self
    }

    pub fn set_url(&mut self, url: Url) {
        self.base_url = url;
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn url(&self, path: &str) -> ClientResult<Url> {
        self.base_url.join(path).map_err(Error::from)
    }

    fn request(&self, method: Method, path: &str) -> ClientResult<reqwest::RequestBuilder> {
        let url = self.url(path)?;
        Ok(self
            .client
            .request(method, url)
            .header(ACCEPT, CONTENT_TYPE)
            .header(AUTHORIZATION, format!("Bearer {}", self.token)))
    }

    async fn check_response(
        method: Method,
        response: reqwest::Response,
    ) -> ClientResult<reqwest::Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        if status == StatusCode::BAD_REQUEST {
            let body = response.text().await?;
            log::trace!("{body}");
            return Err(Error::ValidationErrors(serde_json::from_str(&body)?));
        }

        let url = response.url().clone();
        let body = response.text().await.unwrap_or_default();
        Err(Error::HttpStatusNotSuccess {
            method,
            url,
            status,
            body,
        })
    }

    async fn get<T>(&self, path: &str) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let resp = self.request(Method::GET, path)?.send().await?;
        let resp = Self::check_response(Method::GET, resp).await?;
        Ok(resp.json().await?)
    }

    async fn patch<T>(&self, path: &str, body: &impl Serialize) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let resp = self
            .request(Method::PATCH, path)?
            .header(CONTENT_TYPE_HEADER, CONTENT_TYPE)
            .json(body)
            .send()
            .await?;
        let resp = Self::check_response(Method::PATCH, resp).await?;
        Ok(resp.json().await?)
    }

    async fn post<T>(&self, path: &str, body: Option<&impl Serialize>) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let mut req = self.request(Method::POST, path)?;

        if let Some(body) = body {
            req = req.header(CONTENT_TYPE_HEADER, CONTENT_TYPE).json(body);
        }

        let resp = req.send().await?;
        let resp = Self::check_response(Method::POST, resp).await?;
        Ok(resp.json().await?)
    }

    async fn delete(&self, path: &str) -> ClientResult {
        let resp = self.request(Method::DELETE, path)?.send().await?;
        Self::check_response(Method::DELETE, resp).await?;
        Ok(())
    }

    pub async fn accounts(&self) -> ClientResult<Vec<Account>> {
        self.get("api/accounts").await
    }

    pub async fn rename_account(&self, account_id: Uuid, new_name: &str) -> ClientResult<Account> {
        self.patch(
            &format!("api/accounts/{account_id}"),
            &json!({ "name": new_name }),
        )
        .await
    }

    pub async fn aggregator(&self, aggregator_id: Uuid) -> ClientResult<Aggregator> {
        self.get(&format!("api/aggregators/{aggregator_id}")).await
    }

    pub async fn aggregators(&self, account_id: Uuid) -> ClientResult<Vec<Aggregator>> {
        self.get(&format!("api/accounts/{account_id}/aggregators"))
            .await
    }

    pub async fn create_aggregator(
        &self,
        account_id: Uuid,
        aggregator: NewAggregator,
    ) -> ClientResult<Aggregator> {
        self.post(
            &format!("api/accounts/{account_id}/aggregators"),
            Some(&aggregator),
        )
        .await
    }

    pub async fn rename_aggregator(
        &self,
        aggregator_id: Uuid,
        new_name: &str,
    ) -> ClientResult<Aggregator> {
        self.patch(
            &format!("api/aggregators/{aggregator_id}"),
            &json!({ "name": new_name }),
        )
        .await
    }

    pub async fn rotate_aggregator_bearer_token(
        &self,
        aggregator_id: Uuid,
        new_bearer_token: &str,
    ) -> ClientResult<Aggregator> {
        self.patch(
            &format!("api/aggregators/{aggregator_id}"),
            &json!({ "bearer_token": new_bearer_token }),
        )
        .await
    }

    pub async fn update_aggregator_configuration(
        &self,
        aggregator_id: Uuid,
    ) -> ClientResult<Aggregator> {
        self.patch(&format!("api/aggregators/{aggregator_id}"), &json!({}))
            .await
    }

    pub async fn delete_aggregator(&self, aggregator_id: Uuid) -> ClientResult {
        self.delete(&format!("api/aggregators/{aggregator_id}"))
            .await
    }

    pub async fn memberships(&self, account_id: Uuid) -> ClientResult<Vec<Membership>> {
        self.get(&format!("api/accounts/{account_id}/memberships"))
            .await
    }

    pub async fn delete_membership(&self, membership_id: Uuid) -> ClientResult {
        self.delete(&format!("api/memberships/{membership_id}"))
            .await
    }

    pub async fn create_membership(
        &self,
        account_id: Uuid,
        email: &str,
    ) -> ClientResult<Membership> {
        self.post(
            &format!("api/accounts/{account_id}/memberships"),
            Some(&json!({ "user_email": email })),
        )
        .await
    }

    pub async fn tasks(&self, account_id: Uuid) -> ClientResult<Vec<Task>> {
        self.get(&format!("api/accounts/{account_id}/tasks")).await
    }

    pub async fn task(&self, task_id: &str) -> ClientResult<Task> {
        self.get(&format!("api/tasks/{task_id}")).await
    }

    pub async fn create_task(&self, account_id: Uuid, task: NewTask) -> ClientResult<Task> {
        self.post(&format!("api/accounts/{account_id}/tasks"), Some(&task))
            .await
    }

    pub async fn rename_task(&self, task_id: &str, new_name: &str) -> ClientResult<Task> {
        self.patch(&format!("api/tasks/{task_id}"), &json!({"name": new_name}))
            .await
    }

    pub async fn set_task_expiration(
        &self,
        task_id: &str,
        expiration: Option<&OffsetDateTime>,
    ) -> ClientResult<Task> {
        self.patch(
            &format!("api/tasks/{task_id}"),
            &json!({
                "expiration": expiration.map(|e| e.format(&Rfc3339)).transpose()?
            }),
        )
        .await
    }

    pub async fn delete_task(&self, task_id: &str) -> ClientResult<()> {
        self.delete(&format!("api/tasks/{task_id}")).await
    }

    pub async fn force_delete_task(&self, task_id: &str) -> ClientResult<()> {
        self.delete(&format!("api/tasks/{task_id}?force=true"))
            .await
    }

    pub async fn api_tokens(&self, account_id: Uuid) -> ClientResult<Vec<ApiToken>> {
        self.get(&format!("api/accounts/{account_id}/api_tokens"))
            .await
    }

    pub async fn create_api_token(&self, account_id: Uuid) -> ClientResult<ApiToken> {
        self.post(
            &format!("api/accounts/{account_id}/api_tokens"),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn delete_api_token(&self, api_token_id: Uuid) -> ClientResult {
        self.delete(&format!("api/api_tokens/{api_token_id}")).await
    }

    pub async fn collector_credentials(
        &self,
        account_id: Uuid,
    ) -> ClientResult<Vec<CollectorCredential>> {
        self.get(&format!("api/accounts/{account_id}/collector_credentials"))
            .await
    }

    pub async fn rename_collector_credential(
        &self,
        collector_credential_id: Uuid,
        new_name: &str,
    ) -> ClientResult<CollectorCredential> {
        self.patch(
            &format!("api/collector_credentials/{collector_credential_id}"),
            &json!({"name": new_name}),
        )
        .await
    }

    pub async fn create_collector_credential(
        &self,
        account_id: Uuid,
        hpke_config: &HpkeConfig,
        name: Option<&str>,
    ) -> ClientResult<CollectorCredential> {
        self.post(
            &format!("api/accounts/{account_id}/collector_credentials"),
            Some(&json!({
                "name": name,
                "hpke_config": STANDARD.encode(hpke_config.get_encoded()?)
            })),
        )
        .await
    }

    pub async fn delete_collector_credential(&self, collector_credential_id: Uuid) -> ClientResult {
        self.delete(&format!(
            "api/collector_credentials/{collector_credential_id}"
        ))
        .await
    }

    pub async fn shared_aggregators(&self) -> ClientResult<Vec<Aggregator>> {
        self.get("api/aggregators").await
    }
}

#[cfg(feature = "admin")]
impl DivviupClient {
    pub async fn create_account(&self, name: &str) -> ClientResult<Account> {
        self.post("api/accounts", Some(&json!({ "name": name })))
            .await
    }

    pub async fn create_shared_aggregator(
        &self,
        aggregator: NewSharedAggregator,
    ) -> ClientResult<Aggregator> {
        self.post("api/aggregators", Some(&aggregator)).await
    }
}

pub type ClientResult<T = ()> = Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unexpected http status {method} {url} {status}: {body}")]
    HttpStatusNotSuccess {
        method: Method,
        url: Url,
        status: StatusCode,
        body: String,
    },

    #[error("Validation errors:\n{0}")]
    ValidationErrors(ValidationErrors),

    #[error(transparent)]
    Codec(#[from] CodecError),

    #[error("time formatting error: {0}")]
    TimeFormat(#[from] time::error::Format),
}
