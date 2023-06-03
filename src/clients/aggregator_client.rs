use crate::{
    clients::{ClientConnExt, ClientError},
    ApiConfig,
};
use mini_moka::sync::Cache;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use trillium::{HeaderValue, KnownHeaderName, Method};
use trillium_api::FromConn;
use trillium_client::{Client, Conn};
use url::Url;
pub mod api_types;
pub use api_types::{TaskCreate, TaskIds, TaskMetrics, TaskResponse};

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client,
    base_url: Url,
    auth_header: HeaderValue,
    metrics_cache: Cache<String, TaskMetrics>,
}

#[trillium::async_trait]
impl FromConn for AggregatorClient {
    async fn from_conn(conn: &mut trillium::Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

impl AggregatorClient {
    pub fn new(config: &ApiConfig) -> Self {
        Self {
            client: config.client.clone(),
            base_url: config.aggregator_api_url.clone(),
            auth_header: HeaderValue::from(format!("Bearer {}", config.aggregator_secret.clone())),
            metrics_cache: Cache::builder()
                .time_to_live(Duration::from_secs(30 * 60))
                .build(),
        }
    }

    pub async fn get_task_ids(&self) -> Result<Vec<String>, ClientError> {
        let mut ids = vec![];
        let mut path = String::from("/task_ids");
        loop {
            let TaskIds {
                task_ids,
                pagination_token,
            } = self.get(&path).await?;

            ids.extend(task_ids);

            match pagination_token {
                Some(pagination_token) => {
                    path = format!("/task_ids?pagination_token={pagination_token}");
                }
                None => break Ok(ids),
            }
        }
    }

    pub async fn get_task_metrics(&self, task_id: &str) -> Result<TaskMetrics, ClientError> {
        let task_id = task_id.to_string();
        if let Some(metrics) = self.metrics_cache.get(&task_id) {
            log::trace!("using metrict from cache for {task_id}");
            return Ok(metrics);
        }
        let metrics: TaskMetrics = self.get(&format!("/tasks/{task_id}/metrics")).await?;
        self.metrics_cache.insert(task_id, metrics);
        Ok(metrics)
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), ClientError> {
        self.delete(&format!("/tasks/{task_id}")).await
    }

    pub async fn create_task(&self, task_create: TaskCreate) -> Result<TaskResponse, ClientError> {
        self.post("/tasks", &task_create).await
    }

    // private below here

    fn url(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }

    fn conn(&self, method: Method, path: &str) -> Conn {
        self.client
            .build_conn(method, self.url(path))
            .with_header(KnownHeaderName::Authorization, self.auth_header.clone())
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
