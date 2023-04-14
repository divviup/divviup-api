use crate::{
    clients::{expect_ok, Client, ClientError, Conn},
    entity::NewTask,
    ApiConfig,
};
use trillium::{HeaderValue, KnownHeaderName, Method};
use trillium_api::FromConn;
use url::Url;

pub mod api_types;
pub use api_types::{TaskCreate, TaskIds, TaskMetrics, TaskResponse};

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client,
    base_url: Url,
    auth_header: HeaderValue,
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
            client: Client::new().with_default_pool(),
            base_url: config.aggregator_url.clone(),
            auth_header: HeaderValue::from(format!("Bearer {}", config.aggregator_secret.clone())),
        }
    }

    fn url(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }

    fn conn(&self, method: Method, path: &str) -> Conn<'_> {
        self.client
            .build_conn(method, self.url(path))
            .with_header(KnownHeaderName::Authorization, self.auth_header.clone())
    }

    fn get(&self, path: &str) -> Conn<'_> {
        self.conn(Method::Get, path)
    }

    fn post(&self, path: &str) -> Conn<'_> {
        self.conn(Method::Post, path)
    }

    fn delete(&self, path: &str) -> Conn<'_> {
        self.conn(Method::Delete, path)
    }

    pub async fn health_check(&self) -> Result<(), ClientError> {
        let mut conn = self.get("/health").await?;
        expect_ok(&mut conn).await?;
        Ok(())
    }

    pub async fn get_task_ids(&self) -> Result<Vec<String>, ClientError> {
        let mut ids = vec![];
        let mut path = String::from("/task_ids");
        loop {
            let mut conn = self.get(&path).await?;
            expect_ok(&mut conn).await?;

            let TaskIds {
                task_ids,
                pagination_token,
            } = conn.response_json().await?;

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
        let mut conn = self.get(&format!("/tasks/{task_id}/metrics")).await?;
        expect_ok(&mut conn).await?;
        Ok(conn.response_json().await?)
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), ClientError> {
        let mut conn = self.delete(&format!("/tasks/{task_id}")).await?;
        expect_ok(&mut conn).await?;
        Ok(())
    }

    pub async fn create_task(&self, task: NewTask) -> Result<TaskResponse, ClientError> {
        let mut conn = self
            .post("/tasks")
            .with_json_body(&TaskCreate::from(task))?
            .await?;
        expect_ok(&mut conn).await?;
        Ok(conn.response_json().await?)
    }
}
