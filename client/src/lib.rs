mod account;
mod aggregator;
mod api_token;
mod membership;
mod task;

const CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::{future::Future, pin::Pin};
use trillium_http::{HeaderValue, Headers, KnownHeaderName, Method};

pub use account::Account;
pub use aggregator::{Aggregator, NewAggregator, Role};
pub use api_token::ApiToken;
pub use membership::Membership;
pub use task::{NewTask, Task, Vdaf};
pub use time::OffsetDateTime;
pub use trillium_client::Client;
pub use trillium_client::Conn;
pub use url::Url;
pub use uuid::Uuid;

trait ErrInto<T, E1, E2> {
    fn err_into(self) -> Result<T, E2>;
}
impl<T, E1, E2> ErrInto<T, E1, E2> for Result<T, E1>
where
    E2: From<E1>,
{
    fn err_into(self) -> Result<T, E2> {
        self.map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct DivviupClient {
    http_client: Client,
    headers: Headers,
    url: Url,
}

const DEFAULT_URL: &str = "https://api.staging.divviup.org/";

impl DivviupClient {
    pub fn new(token: String, http_client: impl Into<Client>) -> Self {
        let headers = Headers::from_iter([
            (KnownHeaderName::Accept, HeaderValue::from(CONTENT_TYPE)),
            (
                KnownHeaderName::Authorization,
                HeaderValue::from(format!("Bearer {token}")),
            ),
        ]);

        Self {
            url: DEFAULT_URL.parse().unwrap(),
            http_client: http_client.into(),
            headers,
        }
    }

    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }

    fn url(&self, path: &str) -> Url {
        self.url.join(path).unwrap()
    }

    fn conn(&self, method: Method, path: &str) -> Conn {
        self.http_client
            .build_conn(method, self.url(path))
            .with_headers(self.headers.clone())
    }

    async fn get<T>(&self, path: &str) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        self.conn(Method::Get, path)
            .success_or_error()
            .await?
            .response_json()
            .await
            .err_into()
    }

    async fn patch<T>(&self, path: &str, body: &impl Serialize) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        self.conn(Method::Patch, path)
            .with_json_body(body)?
            .with_header(KnownHeaderName::ContentType, CONTENT_TYPE)
            .success_or_error()
            .await?
            .response_json()
            .await
            .err_into()
    }

    async fn post<T>(&self, path: &str, body: Option<&impl Serialize>) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let mut conn = self.conn(Method::Post, path);

        if let Some(body) = body {
            conn = conn
                .with_json_body(body)?
                .with_header(KnownHeaderName::ContentType, CONTENT_TYPE);
        }

        conn.success_or_error()
            .await?
            .response_json()
            .await
            .err_into()
    }

    async fn delete(&self, path: &str) -> ClientResult {
        let _ = self.conn(Method::Delete, path).success_or_error().await?;
        Ok(())
    }

    pub async fn accounts(&self) -> ClientResult<Vec<Account>> {
        self.get("api/accounts").await
    }

    pub async fn create_account(&self, name: &str) -> ClientResult<Account> {
        self.post("api/accounts", Some(&json!({ "name": name })))
            .await
    }

    pub async fn rename_account(&self, account_id: Uuid, new_name: &str) -> ClientResult<Account> {
        self.patch(
            &format!("api/accounts/{account_id}"),
            &json!({ "name": new_name }),
        )
        .await
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
    ) -> ClientResult<Account> {
        self.patch(
            &format!("api/aggregators/{aggregator_id}"),
            &json!({ "bearer_token": new_bearer_token }),
        )
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

    pub async fn create_task(&self, account_id: Uuid, task: NewTask) -> ClientResult<Task> {
        self.post(&format!("api/accounts/{account_id}/tasks"), Some(&task))
            .await
    }

    pub async fn task_collector_auth_tokens(&self, task_id: &str) -> ClientResult<Vec<String>> {
        self.get(&format!("api/tasks/{task_id}/collector_auth_tokens"))
            .await
    }

    pub async fn rename_task(&self, task_id: &str, new_name: &str) -> ClientResult<Task> {
        self.patch(&format!("api/tasks/{task_id}"), &json!({"name": new_name}))
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
}

pub type ClientResult<T = ()> = Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] trillium_http::Error),

    #[error(transparent)]
    Client(#[from] trillium_client::ClientSerdeError),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("unexpected http status {method} {url} {status:?}: {body}")]
    HttpStatusNotSuccess {
        method: Method,
        url: Url,
        status: Option<trillium_http::Status>,
        body: String,
    },
}

pub trait ClientConnExt: Sized {
    fn success_or_error(self)
        -> Pin<Box<dyn Future<Output = ClientResult<Self>> + Send + 'static>>;
}
impl ClientConnExt for Conn {
    fn success_or_error(
        self,
    ) -> Pin<Box<dyn Future<Output = ClientResult<Self>> + Send + 'static>> {
        Box::pin(async move {
            match self.await?.success() {
                Ok(conn) => Ok(conn),
                Err(mut error) => {
                    let status = error.status();
                    let url = error.url().clone();
                    let method = error.method();
                    let body = error.response_body().await?;
                    Err(Error::HttpStatusNotSuccess {
                        method,
                        url,
                        status,
                        body,
                    })
                }
            }
        })
    }
}
