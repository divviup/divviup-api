use crate::{
    entity::{
        aggregator::{
            Features, QueryTypeName, QueryTypeNameSet, Role as AggregatorRole, VdafNameSet,
        },
        task::vdaf::{BucketLength, ContinuousBuckets, CountVec, Histogram, Sum, SumVec, Vdaf},
        Aggregator, Protocol, ProvisionableTask, Task,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[non_exhaustive]
pub enum AggregatorVdaf {
    Prio3Count,
    Prio3Sum {
        bits: u8,
    },
    Prio3Histogram(HistogramType),
    Prio3CountVec {
        length: u64,
        chunk_length: Option<u64>,
    },
    Prio3SumVec {
        bits: u8,
        length: u64,
        chunk_length: Option<u64>,
    },
}

impl PartialEq<Vdaf> for AggregatorVdaf {
    fn eq(&self, other: &Vdaf) -> bool {
        other.eq(self)
    }
}

impl PartialEq<AggregatorVdaf> for Vdaf {
    fn eq(&self, other: &AggregatorVdaf) -> bool {
        match (self, other) {
            (Vdaf::Count, AggregatorVdaf::Prio3Count) => true,
            (
                Vdaf::Histogram(histogram),
                AggregatorVdaf::Prio3Histogram(HistogramType::Opaque {
                    length,
                    chunk_length,
                }),
            ) => histogram.length() == *length && histogram.chunk_length() == *chunk_length,
            (
                Vdaf::Histogram(Histogram::Continuous(ContinuousBuckets {
                    buckets: Some(lhs_buckets),
                    chunk_length: lhs_chunk_length,
                })),
                AggregatorVdaf::Prio3Histogram(HistogramType::Buckets {
                    buckets: rhs_buckets,
                    chunk_length: rhs_chunk_length,
                }),
            ) => lhs_buckets == rhs_buckets && lhs_chunk_length == rhs_chunk_length,
            (Vdaf::Sum(Sum { bits: Some(lhs) }), AggregatorVdaf::Prio3Sum { bits: rhs }) => {
                lhs == rhs
            }
            (
                Vdaf::CountVec(CountVec {
                    length: Some(lhs_length),
                    chunk_length: lhs_chunk_length,
                }),
                AggregatorVdaf::Prio3CountVec {
                    length: rhs_length,
                    chunk_length: rhs_chunk_length,
                },
            ) => lhs_length == rhs_length && lhs_chunk_length == rhs_chunk_length,
            (
                Vdaf::SumVec(SumVec {
                    bits: Some(lhs_bits),
                    length: Some(lhs_length),
                    chunk_length: lhs_chunk_length,
                }),
                AggregatorVdaf::Prio3SumVec {
                    bits: rhs_bits,
                    length: rhs_length,
                    chunk_length: rhs_chunk_length,
                },
            ) => {
                lhs_bits == rhs_bits
                    && lhs_length == rhs_length
                    && lhs_chunk_length == rhs_chunk_length
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum HistogramType {
    Opaque {
        length: u64,
        chunk_length: Option<u64>,
    },
    Buckets {
        buckets: Vec<u64>,
        chunk_length: Option<u64>,
    },
}

impl From<AggregatorVdaf> for Vdaf {
    fn from(value: AggregatorVdaf) -> Self {
        match value {
            AggregatorVdaf::Prio3Count => Self::Count,
            AggregatorVdaf::Prio3Sum { bits } => Self::Sum(Sum { bits: Some(bits) }),
            AggregatorVdaf::Prio3Histogram(HistogramType::Buckets {
                buckets,
                chunk_length,
            }) => Self::Histogram(Histogram::Continuous(ContinuousBuckets {
                buckets: Some(buckets),
                chunk_length,
            })),
            AggregatorVdaf::Prio3Histogram(HistogramType::Opaque {
                length,
                chunk_length,
            }) => Self::Histogram(Histogram::Opaque(BucketLength {
                length,
                chunk_length,
            })),
            AggregatorVdaf::Prio3CountVec {
                length,
                chunk_length,
            } => Self::CountVec(CountVec {
                length: Some(length),
                chunk_length,
            }),
            AggregatorVdaf::Prio3SumVec {
                bits,
                length,
                chunk_length,
            } => Self::SumVec(SumVec {
                length: Some(length),
                bits: Some(bits),
                chunk_length,
            }),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryType {
    TimeInterval,
    FixedSize {
        max_batch_size: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        batch_time_window_size: Option<u64>,
    },
}

impl QueryType {
    pub fn name(&self) -> QueryTypeName {
        match self {
            QueryType::TimeInterval => QueryTypeName::TimeInterval,
            QueryType::FixedSize { .. } => QueryTypeName::FixedSize,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum AuthenticationToken {
    /// Type of the authentication token. Authentication token type is always "Bearer" in
    /// divviup-api.
    Bearer {
        /// Encoded value of the token. The encoding is opaque to divviup-api.
        token: String,
    },
}

impl AuthenticationToken {
    pub fn new(token: String) -> Self {
        Self::Bearer { token }
    }

    pub fn token(self) -> String {
        match self {
            Self::Bearer { token } => token,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum AuthenticationTokenHash {
    /// Type of the authentication token hash. Authentication token hash type is always "Bearer" in
    /// divviup-api.
    Bearer {
        /// Encoded value of the token hash. The encoding is opaque to divviup-api.
        hash: String,
    },
}

impl AuthenticationTokenHash {
    pub fn new(hash: String) -> Self {
        Self::Bearer { hash }
    }

    pub fn hash(self) -> String {
        match self {
            Self::Bearer { hash } => hash,
        }
    }
}

impl AsRef<str> for AuthenticationTokenHash {
    fn as_ref(&self) -> &str {
        match self {
            Self::Bearer { ref hash } => hash.as_str(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_auth_token: Option<AuthenticationToken>,
    pub peer_aggregator_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: AggregatorVdaf,
    pub role: Role,
    pub max_batch_query_count: u64,
    pub task_expiration: Option<JanusTime>,
    pub min_batch_size: u64,
    pub time_precision: u64,
    pub collector_hpke_config: HpkeConfig,
    pub vdaf_verify_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collector_auth_token_hash: Option<AuthenticationTokenHash>,
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

        let collector_auth_token_hash =
            if role == Role::Leader && target_aggregator.features.token_hash_enabled() {
                new_task
                    .collector_credential
                    .token_hash
                    .clone()
                    .map(|token| AuthenticationTokenHash::Bearer { hash: token })
            } else {
                None
            };

        Ok(Self {
            peer_aggregator_endpoint: if role == Role::Leader {
                new_task.helper_aggregator.dap_url.clone().into()
            } else {
                new_task.leader_aggregator.dap_url.clone().into()
            },
            query_type: new_task.query_type(),
            vdaf: new_task.aggregator_vdaf.clone(),
            role,
            max_batch_query_count: 1,
            task_expiration: new_task.expiration.map(|expiration| {
                JanusTime::from_seconds_since_epoch(expiration.unix_timestamp().try_into().unwrap())
            }),
            min_batch_size: new_task.min_batch_size,
            time_precision: new_task.time_precision_seconds,
            collector_hpke_config: new_task.collector_credential.hpke_config().clone(),
            vdaf_verify_key: new_task.vdaf_verify_key.clone(),
            aggregator_auth_token: new_task
                .aggregator_auth_token
                .clone()
                .map(AuthenticationToken::new),
            collector_auth_token_hash,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TaskPatch {
    pub task_expiration: Option<JanusTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: TaskId,
    pub peer_aggregator_endpoint: Url,
    pub query_type: QueryType,
    pub vdaf: AggregatorVdaf,
    pub role: Role,
    pub vdaf_verify_key: String,
    pub max_batch_query_count: u64,
    pub task_expiration: Option<JanusTime>,
    pub report_expiry_age: Option<JanusDuration>,
    pub min_batch_size: u64,
    pub time_precision: JanusDuration,
    pub tolerable_clock_skew: JanusDuration,
    pub collector_hpke_config: HpkeConfig,
    pub aggregator_auth_token: Option<AuthenticationToken>,
    pub collector_auth_token: Option<AuthenticationToken>,
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaskUploadMetrics {
    /// Reports that fell into a time interval that had already been collected.
    pub interval_collected: u64,
    /// Reports that could not be decoded.
    pub report_decode_failure: u64,
    /// Reports that could not be decrypted.
    pub report_decrypt_failure: u64,
    /// Reports that contained a timestamp too far in the past.
    pub report_expired: u64,
    /// Reports that were encrypted with an old or unknown HPKE key.
    pub report_outdated_key: u64,
    /// Reports that were successfully uploaded.
    pub report_success: u64,
    /// Reports that contain a timestamp too far in the future.
    pub report_too_early: u64,
    /// Reports that were submitted to the task after the task's expiry.
    pub task_expired: u64,
}

impl PartialEq<Task> for TaskUploadMetrics {
    fn eq(&self, other: &Task) -> bool {
        other.report_counter_interval_collected == self.interval_collected as i64
            && other.report_counter_decode_failure == self.report_decode_failure as i64
            && other.report_counter_decrypt_failure == self.report_decrypt_failure as i64
            && other.report_counter_expired == self.report_expired as i64
            && other.report_counter_outdated_key == self.report_outdated_key as i64
            && other.report_counter_success == self.report_success as i64
            && other.report_counter_too_early == self.report_too_early as i64
            && other.report_counter_task_expired == self.task_expired as i64
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AggregatorApiConfig {
    pub dap_url: Url,
    pub role: AggregatorRole,
    pub vdafs: VdafNameSet,
    pub query_types: QueryTypeNameSet,
    pub protocol: Protocol,
    #[serde(default)]
    pub features: Features,
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
      "length": 5,
      "chunk_length": null
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
      "length": 5,
      "chunk_length": null
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
  "aggregator_auth_token": {
    "type": "Bearer",
    "token": "YWdncmVnYXRvci0xMjM0NTY3OA"
  },
  "collector_auth_token": {
    "type": "Bearer",
    "token": "Y29sbGVjdG9yLWFiY2RlZjAw"
  },
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
