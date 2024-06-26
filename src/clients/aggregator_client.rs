use crate::{
    clients::{ClientConnExt, ClientError},
    entity::{task::ProvisionableTask, Aggregator},
    handler::Error,
};
use janus_messages::Time as JanusTime;
use serde::{de::DeserializeOwned, Serialize};
use trillium_client::{Client, KnownHeaderName};
use url::Url;
pub mod api_types;
pub use api_types::{
    AggregatorApiConfig, TaskCreate, TaskIds, TaskPatch, TaskResponse, TaskUploadMetrics,
};

const CONTENT_TYPE: &str = "application/vnd.janus.aggregator+json;version=0.1";

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client,
    aggregator: Aggregator,
}

impl AsRef<Client> for AggregatorClient {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}

impl AggregatorClient {
    pub fn new(client: Client, aggregator: Aggregator, bearer_token: &str) -> Self {
        let client = client
            .with_base(aggregator.api_url.clone())
            .with_default_header(
                KnownHeaderName::Authorization,
                format!("Bearer {bearer_token}"),
            )
            .with_default_header(KnownHeaderName::Accept, CONTENT_TYPE);

        Self { client, aggregator }
    }

    pub async fn get_config(
        client: Client,
        base_url: Url,
        token: &str,
    ) -> Result<AggregatorApiConfig, ClientError> {
        client
            .get(base_url)
            .with_request_header(KnownHeaderName::Authorization, format!("Bearer {token}"))
            .with_request_header(KnownHeaderName::Accept, CONTENT_TYPE)
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

    pub async fn get_task_upload_metrics(
        &self,
        task_id: &str,
    ) -> Result<TaskUploadMetrics, ClientError> {
        self.get(&format!("tasks/{task_id}/metrics/uploads")).await
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

    // private below here

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        self.client
            .get(path)
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
        self.client
            .post(path)
            .with_json_body(body)?
            .with_request_header(KnownHeaderName::ContentType, CONTENT_TYPE)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }

    async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ClientError> {
        self.client
            .patch(path)
            .with_json_body(body)?
            .with_request_header(KnownHeaderName::ContentType, CONTENT_TYPE)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }
}
