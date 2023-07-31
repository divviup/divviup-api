use crate::{
    clients::{ClientConnExt, ClientError},
    entity::{task::ProvisionableTask, Aggregator},
    handler::Error,
};
use serde::{de::DeserializeOwned, Serialize};
use trillium::{HeaderValue, KnownHeaderName, Method};
use trillium_client::{Client, Conn};
use url::Url;
pub mod api_types;
pub use api_types::{AggregatorApiConfig, TaskCreate, TaskIds, TaskMetrics, TaskResponse};

const CONTENT_TYPE: &str = "application/vnd.janus.aggregator+json;version=0.1";

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client,
    base_url: Url,
    auth_header: HeaderValue,
    aggregator: Aggregator,
}

impl AsRef<Client> for AggregatorClient {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}

impl AggregatorClient {
    pub fn new(client: Client, aggregator: Aggregator) -> Self {
        let mut base_url: Url = aggregator.api_url.clone().into();
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        Self {
            client,
            base_url,
            auth_header: HeaderValue::from(format!("Bearer {}", &aggregator.bearer_token)),
            aggregator,
        }
    }

    pub async fn get_config(
        client: Client,
        base_url: Url,
        token: &str,
    ) -> Result<AggregatorApiConfig, ClientError> {
        client
            .get(base_url)
            .with_header(KnownHeaderName::Authorization, format!("Bearer {token}"))
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_task_ids(&self) -> Result<Vec<String>, ClientError> {
        let mut ids = vec![];
        let mut path = String::from("task_ids");
        loop {
            let TaskIds {
                task_ids,
                pagination_token,
            } = self.get(&path).await?;

            ids.extend(task_ids);

            match pagination_token {
                Some(pagination_token) => {
                    path = format!("task_ids?pagination_token={pagination_token}");
                }
                None => break Ok(ids),
            }
        }
    }

    pub async fn get_task(&self, task_id: &str) -> Result<TaskResponse, ClientError> {
        self.get(&format!("tasks/{task_id}")).await
    }

    pub async fn get_task_metrics(&self, task_id: &str) -> Result<TaskMetrics, ClientError> {
        self.get(&format!("tasks/{task_id}/metrics")).await
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), ClientError> {
        self.delete(&format!("tasks/{task_id}")).await
    }

    pub async fn create_task(&self, task: &ProvisionableTask) -> Result<TaskResponse, Error> {
        let task_create = TaskCreate::build(&self.aggregator, task)?;
        self.post("tasks", &task_create).await.map_err(Into::into)
    }

    // private below here

    fn url(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }

    fn conn(&self, method: Method, path: &str) -> Conn {
        self.client
            .build_conn(method, self.url(path))
            .with_header(KnownHeaderName::Authorization, self.auth_header.clone())
            .with_header(KnownHeaderName::Accept, CONTENT_TYPE)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        self.conn(Method::Get, path)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ClientError> {
        self.conn(Method::Post, path)
            .with_json_body(body)?
            .with_header(KnownHeaderName::ContentType, CONTENT_TYPE)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }

    async fn delete(&self, path: &str) -> Result<(), ClientError> {
        let _ = self
            .conn(Method::Delete, path)
            .success_or_client_error()
            .await?;
        Ok(())
    }
}
