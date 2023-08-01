use crate::{
    entity::{
        aggregator::{QueryTypeName, QueryTypeNameSet, Role as AggregatorRole, VdafNameSet},
        task::vdaf::{CountVec, Histogram, Sum, SumVec, Vdaf},
        Aggregator, ProvisionableTask,
    },
    handler::Error,
};
use serde::{Deserialize, Serialize};
use time::{error::ComponentRange, OffsetDateTime};
use url::Url;

pub use janus_messages::{
    codec::{Decode, Encode},
    Duration as JanusDuration, HpkeAeadId, HpkeConfig, HpkeConfigId, HpkeConfigList, HpkeKdfId,
    HpkeKemId, HpkePublicKey, Role, TaskId, Time as JanusTime,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VdafInstance {
    Prio3Count,
    Prio3Sum { bits: u8 },
    Prio3Histogram { buckets: Vec<u64> },
    Prio3CountVec { length: u64 },
    Prio3SumVec { bits: u8, length: u64 },
}

impl From<VdafInstance> for Vdaf {
    fn from(value: VdafInstance) -> Self {
        match value {
            VdafInstance::Prio3Count => Self::Count,
            VdafInstance::Prio3Sum { bits } => Self::Sum(Sum { bits: Some(bits) }),
            VdafInstance::Prio3Histogram { buckets } => Self::Histogram(Histogram {
                buckets: Some(buckets),
            }),
            VdafInstance::Prio3CountVec { length } => Self::CountVec(CountVec {
                length: Some(length),
            }),
            VdafInstance::Prio3SumVec { bits, length } => Self::SumVec(SumVec {
                length: Some(length),
                bits: Some(bits),
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
            Vdaf::CountVec(CountVec { length }) => Self::Prio3CountVec {
                length: length.unwrap(),
            },
            Vdaf::SumVec(SumVec { length, bits }) => Self::Prio3SumVec {
                bits: bits.unwrap(),
                length: length.unwrap(),
            },
            Vdaf::Unrecognized => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryType {
    TimeInterval,
    FixedSize { max_batch_size: u64 },
}

impl QueryType {
    pub fn name(&self) -> QueryTypeName {
        match self {
            QueryType::TimeInterval => QueryTypeName::TimeInterval,
            QueryType::FixedSize { .. } => QueryTypeName::FixedSize,
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collector_auth_token: Option<String>,
    pub peer_aggregator_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub max_batch_query_count: u64,
    pub task_expiration: Option<JanusTime>,
    pub min_batch_size: u64,
    pub time_precision: u64,
    pub collector_hpke_config: HpkeConfig,
    pub vdaf_verify_key: String,
}

impl TaskCreate {
    pub fn build(
        target_aggregator: &Aggregator,
        new_task: &ProvisionableTask,
    ) -> Result<Self, Error> {
        let role = if new_task.leader_aggregator.id == target_aggregator.id {
            Role::Leader
        } else {
            Role::Helper
        };
        Ok(Self {
            peer_aggregator_endpoint: if role == Role::Leader {
                new_task.helper_aggregator.dap_url.clone().into()
            } else {
                new_task.leader_aggregator.dap_url.clone().into()
            },
            query_type: new_task.max_batch_size.into(),
            vdaf: new_task.vdaf.clone().into(),
            role,
            max_batch_query_count: 1,
            task_expiration: new_task.expiration.map(|expiration| {
                JanusTime::from_seconds_since_epoch(expiration.unix_timestamp().try_into().unwrap())
            }),
            min_batch_size: new_task.min_batch_size,
            time_precision: new_task.time_precision_seconds,
            collector_hpke_config: new_task.hpke_config.clone(),
            vdaf_verify_key: new_task.vdaf_verify_key.clone(),
            aggregator_auth_token: new_task.aggregator_auth_token.clone(),
            collector_auth_token: None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: TaskId,
    pub peer_aggregator_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub vdaf_verify_key: String,
    pub max_batch_query_count: u64,
    pub task_expiration: Option<JanusTime>,
    pub report_expiry_age: Option<JanusDuration>,
    pub min_batch_size: u64,
    pub time_precision: JanusDuration,
    pub tolerable_clock_skew: JanusDuration,
    pub collector_hpke_config: HpkeConfig,
    pub aggregator_auth_token: Option<String>,
    pub collector_auth_token: Option<String>,
    pub aggregator_hpke_configs: Vec<HpkeConfig>,
}

impl TaskResponse {
    pub fn task_expiration(&self) -> Result<Option<OffsetDateTime>, ComponentRange> {
        self.task_expiration
            .map(|t| {
                OffsetDateTime::from_unix_timestamp(t.as_seconds_since_epoch().try_into().unwrap())
            })
            .transpose()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskIds {
    pub task_ids: Vec<String>,
    pub pagination_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskMetrics {
    pub reports: u64,
    pub report_aggregations: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AggregatorApiConfig {
    pub dap_url: Url,
    pub role: AggregatorRole,
    pub vdafs: VdafNameSet,
    pub query_types: QueryTypeNameSet,
}

#[cfg(test)]
mod test {
    use super::{TaskCreate, TaskResponse};

    const TASK_CREATE: &str = r#"{
  "peer_aggregator_endpoint": "https://example.com/",
  "query_type": {
    "FixedSize": {
      "max_batch_size": 999
    }
  },
  "vdaf": {
    "Prio3CountVec": {
      "length": 5
    }
  },
  "role": "Leader",
  "max_batch_query_count": 1,
  "task_expiration": 18446744073709551615,
  "min_batch_size": 100,
  "time_precision": 3600,
  "collector_hpke_config": {
    "id": 7,
    "kem_id": "X25519HkdfSha256",
    "kdf_id": "HkdfSha256",
    "aead_id": "Aes128Gcm",
    "public_key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
  },
  "vdaf_verify_key": "dmRhZiB2ZXJpZnkga2V5IQ"
}"#;

    #[test]
    fn task_create_json_serialization() {
        let task_create: TaskCreate = serde_json::from_str(TASK_CREATE).unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&task_create).unwrap(),
            TASK_CREATE
        );
    }

    const TASK_RESPONSE: &str = r#"{
  "task_id": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "peer_aggregator_endpoint": "https://example.com/",
  "query_type": {
    "FixedSize": {
      "max_batch_size": 999
    }
  },
  "vdaf": {
    "Prio3CountVec": {
      "length": 5
    }
  },
  "role": "Leader",
  "vdaf_verify_key": "dmRhZiB2ZXJpZnkga2V5IQ",
  "max_batch_query_count": 1,
  "task_expiration": 9000000000,
  "report_expiry_age": null,
  "min_batch_size": 100,
  "time_precision": 3600,
  "tolerable_clock_skew": 60,
  "collector_hpke_config": {
    "id": 7,
    "kem_id": "X25519HkdfSha256",
    "kdf_id": "HkdfSha256",
    "aead_id": "Aes128Gcm",
    "public_key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
  },
  "aggregator_auth_token": "YWdncmVnYXRvci0xMjM0NTY3OA",
  "collector_auth_token": "Y29sbGVjdG9yLWFiY2RlZjAw",
  "aggregator_hpke_configs": [
    {
      "id": 13,
      "kem_id": "X25519HkdfSha256",
      "kdf_id": "HkdfSha256",
      "aead_id": "Aes128Gcm",
      "public_key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    }
  ]
}"#;

    #[test]
    fn task_response_json_serialization() {
        let task_response: TaskResponse = serde_json::from_str(TASK_RESPONSE).unwrap();

        assert_eq!(
            task_response.collector_hpke_config.public_key().as_ref(),
            vec![0; 32]
        );

        assert_eq!(
            serde_json::to_string_pretty(&task_response).unwrap(),
            TASK_RESPONSE
        );
    }
}
