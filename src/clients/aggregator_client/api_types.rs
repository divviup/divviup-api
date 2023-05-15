use crate::{
    entity::{
        task::{self, Histogram, Sum, Vdaf},
        NewTask,
    },
    handler::Error,
    ApiConfig,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
pub use janus_messages::{
    Duration as JanusDuration, HpkeAeadId, HpkeConfig, HpkeConfigId, HpkeConfigList, HpkeKdfId,
    HpkeKemId, HpkePublicKey, Role, TaskId, Time as JanusTime,
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

impl TryFrom<task::HpkeConfig> for HpkeConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(value: task::HpkeConfig) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.id.unwrap().into(),
            value.kem_id.unwrap().try_into()?,
            value.kdf_id.unwrap().try_into()?,
            value.aead_id.unwrap().try_into()?,
            value.public_key.unwrap().into_bytes().into(),
        ))
    }
}
impl From<HpkeConfig> for task::HpkeConfig {
    fn from(hpke_config: HpkeConfig) -> Self {
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
    pub aggregator_endpoints: Vec<Url>,
    pub query_type: QueryType,
    pub vdaf: VdafInstance,
    pub role: Role,
    pub max_batch_query_count: u64,
    pub task_expiration: u64,
    pub min_batch_size: u64,
    pub time_precision: u64,
    pub collector_hpke_config: HpkeConfig,
}

impl TaskCreate {
    pub fn build(new_task: NewTask, config: &ApiConfig) -> Result<Self, Error> {
        Ok(Self {
            aggregator_endpoints: if new_task.is_leader.unwrap() {
                vec![
                    config.aggregator_dap_url.clone(),
                    new_task.partner_url.unwrap().parse()?,
                ]
            } else {
                vec![
                    new_task.partner_url.unwrap().parse()?,
                    config.aggregator_dap_url.clone(),
                ]
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
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: TaskId,
    pub aggregator_endpoints: Vec<Url>,
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
    pub collector_hpke_config: HpkeConfig,
    pub aggregator_auth_tokens: Vec<String>,
    pub collector_auth_tokens: Vec<String>,
    pub aggregator_hpke_configs: HashMap<HpkeConfigId, HpkeConfig>,
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
    use serde_json::{from_value, json, to_value};

    use super::{TaskCreate, TaskResponse};

    #[test]
    fn task_create_json_serialization() {
        let task_create_from_janus_aggregator_api_tests = json!({
            "aggregator_endpoints": [
                "http://leader.endpoint/",
                "http://helper.endpoint/"
            ],
            "query_type": "TimeInterval",
            "vdaf": "Prio3Count",
            "role": "Leader",
            "max_batch_query_count": 12,
            "task_expiration": 12345,
            "min_batch_size": 223,
            "time_precision": 62,
            "collector_hpke_config": {
                "id": 199,
                "kem_id": "X25519HkdfSha256",
                "kdf_id": "HkdfSha256",
                "aead_id": "Aes128Gcm",
                "public_key": "p2J0ht1GtUa8XW67AKmYbfzU1L1etPlJiRIiRigzhEw"
            }
        });

        let task_create: TaskCreate =
            from_value(task_create_from_janus_aggregator_api_tests.clone()).unwrap();
        assert_eq!(
            to_value(&task_create).unwrap(),
            task_create_from_janus_aggregator_api_tests
        );
    }

    #[test]
    fn task_response_json_serialization() {
        let task_response_from_janus_aggregator_api_tests = json!({
            "task_id": "NGTX4o1JP4JLUCmM5Vcdl1Mcz41cOGgRnU1V0gU1Z_M",
            "aggregator_endpoints": [
                "http://leader.endpoint/",
                "http://helper.endpoint/"
            ],
            "query_type": "TimeInterval",
            "vdaf": "Prio3Count",
            "role": "Leader",
            "vdaf_verify_keys": [
                "Fvp4ZzHEbJOMGyTjG4Pctw"
            ],
            "max_batch_query_count": 12,
            "task_expiration": 12345,
            "report_expiry_age": 1209600,
            "min_batch_size": 223,
            "time_precision": 62,
            "tolerable_clock_skew": 60,
            "collector_hpke_config": {
                "id": 177,
                "kem_id": "X25519HkdfSha256",
                "kdf_id": "HkdfSha256",
                "aead_id": "Aes128Gcm",
                "public_key": "ifb-I8PBdIwuKcylg2_tRZ2_vf1XOWA-Jx5plLAn52Y"
            },
            "aggregator_auth_tokens": [
                "MTlhMzBiZjE3NWMyN2FlZWFlYTI3NmVjMDIxZDM4MWQ"
            ],
            "collector_auth_tokens": [
                "YzMyYzU4YTc0ZjBmOGU5MjU0YWIzMjA0OGZkMTQyNTE"
            ],
            "aggregator_hpke_configs": {
                "43": {
                    "id": 43,
                    "kem_id": "X25519HkdfSha256",
                    "kdf_id": "HkdfSha256",
                    "aead_id": "Aes128Gcm",
                    "public_key": "j98s3TCKDutLGPFMULsWFgsQc-keIW8WNxp8aMKEJjk"
                }
            }
        });

        let task_response: TaskResponse =
            from_value(task_response_from_janus_aggregator_api_tests.clone()).unwrap();
        assert_eq!(
            to_value(&task_response).unwrap(),
            task_response_from_janus_aggregator_api_tests
        );
    }
}
