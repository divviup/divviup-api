use crate::{
    entity::{
        task::{
            vdaf::{CountVec, Histogram, Sum, SumVec, Vdaf},
            HpkeConfig,
        },
        NewTask,
    },
    handler::Error,
    ApiConfig,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
pub use janus_messages::{
    Duration as JanusDuration, HpkeAeadId, HpkeConfig as JanusHpkeConfig, HpkeConfigId,
    HpkeConfigList, HpkeKdfId, HpkeKemId, HpkePublicKey, Role, TaskId, Time as JanusTime,
};
use serde::{Deserialize, Serialize};
use url::Url;

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

impl TryFrom<HpkeConfig> for JanusHpkeConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: HpkeConfig) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.id.unwrap().into(),
            value.kem_id.unwrap().try_into()?,
            value.kdf_id.unwrap().try_into()?,
            value.aead_id.unwrap().try_into()?,
            URL_SAFE_NO_PAD.decode(value.public_key.unwrap())?.into(),
        ))
    }
}
impl From<JanusHpkeConfig> for HpkeConfig {
    fn from(hpke_config: JanusHpkeConfig) -> Self {
        Self {
            id: Some((*hpke_config.id()).into()),
            kem_id: Some((*hpke_config.kem_id()) as u16),
            kdf_id: Some((*hpke_config.kdf_id()) as u16),
            aead_id: Some((*hpke_config.aead_id()) as u16),
            public_key: Some(URL_SAFE_NO_PAD.encode(hpke_config.public_key())),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vdaf_verify_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collector_auth_token: Option<String>,
    pub leader_endpoint: Url,
    pub helper_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub max_batch_query_count: u64,
    pub task_expiration: u64,
    pub min_batch_size: u64,
    pub time_precision: u64,
    pub collector_hpke_config: JanusHpkeConfig,
}

impl TaskCreate {
    pub fn build(new_task: NewTask, config: &ApiConfig) -> Result<Self, Error> {
        Ok(Self {
            leader_endpoint: if new_task.is_leader.unwrap() {
                config.aggregator_dap_url.clone()
            } else {
                new_task.partner_url.as_deref().unwrap().parse()?
            },
            helper_endpoint: if new_task.is_leader.unwrap() {
                new_task.partner_url.as_deref().unwrap().parse()?
            } else {
                config.aggregator_dap_url.clone()
            },
            query_type: new_task.max_batch_size.into(),
            vdaf: new_task.vdaf.unwrap().into(),
            role: if new_task.is_leader.unwrap() {
                Role::Leader
            } else {
                Role::Helper
            },
            max_batch_query_count: 1,
            task_expiration: new_task
                .expiration
                .map(|task| task.unix_timestamp().try_into().unwrap())
                .unwrap_or(u64::MAX),
            min_batch_size: new_task.min_batch_size.unwrap(),
            time_precision: new_task.time_precision_seconds.unwrap(),
            collector_hpke_config: new_task.hpke_config.unwrap().try_into()?,
            task_id: new_task.id,
            vdaf_verify_key: new_task.vdaf_verify_key,
            aggregator_auth_token: new_task.aggregator_auth_token,
            collector_auth_token: new_task.collector_auth_token,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: TaskId,
    pub leader_endpoint: Url,
    pub helper_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub vdaf_verify_keys: Vec<String>,
    pub max_batch_query_count: u64,
    pub task_expiration: JanusTime,
    pub report_expiry_age: Option<JanusDuration>,
    pub min_batch_size: u64,
    pub time_precision: JanusDuration,
    pub tolerable_clock_skew: JanusDuration,
    pub collector_hpke_config: JanusHpkeConfig,
    pub aggregator_auth_tokens: Vec<String>,
    pub collector_auth_tokens: Vec<String>,
    pub aggregator_hpke_configs: Vec<JanusHpkeConfig>,
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

#[cfg(test)]
mod test {
    use super::{JanusHpkeConfig, TaskCreate, TaskResponse};
    use crate::{aggregator_api_mock::random_hpke_config, entity::task::HpkeConfig};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use validator::Validate;

    const TASK_CREATE: &str = r#"{
  "leader_endpoint": "https://example.com/",
  "helper_endpoint": "https://example.net/",
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
  }
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
  "leader_endpoint": "https://example.com/",
  "helper_endpoint": "https://example.net/",
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
  "vdaf_verify_keys": [
    "dmRhZiB2ZXJpZnkga2V5IQ"
  ],
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
  "aggregator_auth_tokens": [
    "YWdncmVnYXRvci0xMjM0NTY3OA"
  ],
  "collector_auth_tokens": [
    "Y29sbGVjdG9yLWFiY2RlZjAw"
  ],
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

    #[test]
    fn hpke_config_conversion() {
        let janus_hpke_config = random_hpke_config();
        let hpke_config: HpkeConfig = janus_hpke_config.clone().try_into().unwrap();
        assert!(hpke_config.validate().is_ok());
        assert_eq!(
            hpke_config.public_key.as_deref().unwrap(),
            URL_SAFE_NO_PAD.encode(janus_hpke_config.public_key())
        );
        assert_eq!(
            janus_hpke_config,
            JanusHpkeConfig::try_from(hpke_config).unwrap()
        );
    }
}
