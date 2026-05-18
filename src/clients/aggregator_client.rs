use crate::{
    clients::{ClientError, HttpClient, ResponseExt},
    entity::{task::ProvisionableTask, Aggregator},
    handler::Error,
};
use api_types::TaskAggregationJobMetrics;
use axum::http::header;
use janus_messages::Time as JanusTime;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
pub mod api_types;
pub use api_types::{
    AggregatorApiConfig, TaskCreate, TaskIds, TaskPatch, TaskResponse, TaskUploadMetrics,
};

const CONTENT_TYPE: &str = "application/vnd.janus.aggregator+json;version=0.1";

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: HttpClient,
    aggregator: Aggregator,
}

impl AggregatorClient {
    pub fn new(client: HttpClient, aggregator: Aggregator, bearer_token: &str) -> Self {
        let client = client
            .with_base(aggregator.api_url.clone())
            .with_default_header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
            .with_default_header(header::ACCEPT, CONTENT_TYPE);

        Self { client, aggregator }
    }

    pub async fn get_config(
        client: HttpClient,
        base_url: Url,
        token: &str,
    ) -> Result<AggregatorApiConfig, ClientError> {
        client
            .get_url(base_url)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::ACCEPT, CONTENT_TYPE)
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
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

    pub async fn get_task_upload_metrics(
        &self,
        task_id: &str,
    ) -> Result<TaskUploadMetrics, ClientError> {
        self.get(&format!("tasks/{task_id}/metrics/uploads")).await
    }

    pub async fn get_task_aggregation_job_metrics(
        &self,
        task_id: &str,
    ) -> Result<TaskAggregationJobMetrics, ClientError> {
        self.get(&format!("tasks/{task_id}/metrics/aggregations"))
            .await
    }

    pub async fn create_task(&self, task: &ProvisionableTask) -> Result<TaskResponse, Error> {
        let task_create = TaskCreate::build(&self.aggregator, task)?;
        self.post("tasks", &task_create).await.map_err(Into::into)
    }

    pub async fn update_task_expiration(
        &self,
        task_id: &str,
        expiration: Option<JanusTime>,
    ) -> Result<TaskResponse, Error> {
        self.patch(
            &format!("tasks/{task_id}"),
            &TaskPatch {
                task_expiration: expiration,
            },
        )
        .await
        .map_err(Into::into)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        self.client
            .get(path)
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ClientError> {
        self.client
            .post(path)
            .header(header::CONTENT_TYPE, CONTENT_TYPE)
            .json(body)
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ClientError> {
        self.client
            .patch(path)
            .header(header::CONTENT_TYPE, CONTENT_TYPE)
            .json(body)
            .send()
            .await?
            .success_or_client_error()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }
}
