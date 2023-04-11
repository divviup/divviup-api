use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use trillium::{HeaderValue, KnownHeaderName, Method, Status};
use trillium_api::FromConn;
use trillium_client::{Client, ClientSerdeError};
use trillium_rustls::RustlsConnector;
use trillium_tokio::TcpConnector;
use url::Url;

use crate::{
    entity::{
        task::{Histogram, Sum, Vdaf},
        NewTask,
    },
    ApiConfig,
};

type ClientConnector = RustlsConnector<TcpConnector>;
type Conn<'a> = trillium_client::Conn<'a, ClientConnector>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("unexpected api client http status {status:?}: {body}")]
    HttpStatusNotSuccess {
        status: Option<Status>,
        body: String,
    },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] trillium_http::Error),
}

impl From<ClientSerdeError> for ClientError {
    fn from(value: ClientSerdeError) -> Self {
        match value {
            ClientSerdeError::HttpError(h) => h.into(),
            ClientSerdeError::JsonError(j) => j.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client<ClientConnector>,
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

async fn expect_ok(conn: &mut Conn<'_>) -> Result<(), ClientError> {
    if conn.status().map_or(false, |s| s.is_success()) {
        Ok(())
    } else {
        let body = conn.response_body().read_string().await?;
        Err(ClientError::HttpStatusNotSuccess {
            status: conn.status(),
            body,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VdafInstance {
    Prio3Count,
    Prio3Sum { bits: u8 },
    Prio3Histogram { buckets: Vec<i32> },
}

impl From<VdafInstance> for Vdaf {
    fn from(value: VdafInstance) -> Self {
        match value {
            VdafInstance::Prio3Count => Self::Count,
            VdafInstance::Prio3Sum { bits } => Self::Sum(Sum { bits: Some(bits) }),
            VdafInstance::Prio3Histogram { buckets } => Self::Histogram(Histogram {
                buckets: Some(buckets),
            }),
        }
    }
}

impl From<Vdaf> for VdafInstance {
    fn from(value: Vdaf) -> Self {
        match value {
            Vdaf::Count => Self::Prio3Count,
            Vdaf::Histogram(Histogram { buckets }) => Self::Prio3Histogram {
                buckets: buckets.unwrap(),
            },
            Vdaf::Sum(Sum { bits }) => Self::Prio3Sum {
                bits: bits.unwrap(),
            },
            Vdaf::Unrecognized => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Role {
    Collector = 0,
    Client = 1,
    Leader = 2,
    Helper = 3,
}

impl Role {
    pub fn is_leader(&self) -> bool {
        matches!(self, Self::Leader)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HpkeConfig {
    pub id: u8,
    pub kem_id: u8,
    pub kdf_id: u8,
    pub aead_id: u8,
    pub public_key: Vec<u8>,
}

impl From<crate::entity::task::HpkeConfig> for HpkeConfig {
    fn from(value: crate::entity::task::HpkeConfig) -> Self {
        Self {
            id: value.id.unwrap(),
            kem_id: value.kem_id.unwrap(),
            kdf_id: value.kdf_id.unwrap(),
            aead_id: value.aead_id.unwrap(),
            public_key: value.public_key.unwrap().into_bytes(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum QueryType {
    TimeInterval,
    FixedSize { max_batch_size: u64 },
}

impl From<QueryType> for Option<i64> {
    fn from(value: QueryType) -> Self {
        Option::<u64>::from(value).map(|u| u.try_into().unwrap())
    }
}

impl From<QueryType> for Option<u64> {
    fn from(value: QueryType) -> Self {
        match value {
            QueryType::TimeInterval => None,
            QueryType::FixedSize { max_batch_size } => Some(max_batch_size),
        }
    }
}

impl From<Option<u64>> for QueryType {
    fn from(value: Option<u64>) -> Self {
        value.map_or(QueryType::TimeInterval, |max_batch_size| {
            QueryType::FixedSize { max_batch_size }
        })
    }
}

impl From<Option<i64>> for QueryType {
    fn from(value: Option<i64>) -> Self {
        value.map(|i| u64::try_from(i).unwrap()).into()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskCreate {
    pub aggregator_endpoints: Vec<Url>,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub max_batch_query_count: i64,
    pub task_expiration: u64,
    pub min_batch_size: i64,
    pub time_precision: i32,
    pub collector_hpke_config: HpkeConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub aggregator_endpoints: Vec<Url>,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub vdaf_verify_keys: Vec<String>,
    pub max_batch_query_count: i64,
    pub task_expiration: u64,
    pub report_expiry_age: Option<i64>,
    pub min_batch_size: i64,
    pub time_precision: i32,
    pub tolerable_clock_skew: i64,
    pub collector_hpke_config: HpkeConfig,
    pub aggregator_auth_tokens: Vec<String>,
    pub collector_auth_tokens: Vec<String>,
    pub aggregator_hpke_configs: HashMap<u8, HpkeConfig>,
}

impl From<NewTask> for TaskCreate {
    fn from(value: NewTask) -> Self {
        Self {
            aggregator_endpoints: vec![],
            query_type: value.max_batch_size.into(),
            vdaf: value.vdaf.unwrap().into(),
            role: if value.is_leader.unwrap() {
                Role::Leader
            } else {
                Role::Helper
            },
            max_batch_query_count: 0,
            task_expiration: value
                .expiration
                .map(|task| task.unix_timestamp().try_into().unwrap())
                .unwrap_or(u64::MAX),
            min_batch_size: value.min_batch_size.unwrap(),
            time_precision: value.time_precision_seconds.unwrap(),
            collector_hpke_config: value.hpke_config.unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskIds {
    pub task_ids: Vec<String>,
    pub pagination_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct TaskMetrics {
    pub reports: u64,
    pub report_aggregations: u64,
}
