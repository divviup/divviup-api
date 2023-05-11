use crate::entity::{
    task::{self, Histogram, Sum, Vdaf},
    NewTask,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

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

impl From<task::HpkeConfig> for HpkeConfig {
    fn from(value: task::HpkeConfig) -> Self {
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
            max_batch_query_count: 1,
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
