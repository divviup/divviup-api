use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub vdaf: Vdaf,
    pub min_batch_size: u64,
    pub max_batch_size: Option<u64>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    pub time_precision_seconds: u32,
    pub report_count: u32,
    pub aggregate_collection_count: u32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<OffsetDateTime>,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NewTask {
    pub name: String,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
    pub vdaf: Vdaf,
    pub min_batch_size: u64,
    pub max_batch_size: Option<u64>,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub expiration: Option<OffsetDateTime>,
    pub time_precision_seconds: u64,
    #[serde(with = "base64")]
    pub hpke_config: Vec<u8>,
}

mod base64 {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{
        de::{Error, Unexpected, Visitor},
        Deserializer, Serializer,
    };

    pub fn serialize<S: Serializer>(
        token_hash: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(token_hash))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        struct Base64Visitor;
        impl<'de> Visitor<'de> for Base64Visitor {
            type Value = Vec<u8>;
            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "base64")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                STANDARD
                    .decode(v)
                    .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(Base64Visitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Vdaf {
    #[serde(rename = "count")]
    Count,

    #[serde(rename = "histogram")]
    Histogram { buckets: Vec<u64> },

    #[serde(rename = "sum")]
    Sum { bits: u8 },

    #[serde(rename = "count_vec")]
    CountVec { length: u64 },

    #[serde(rename = "sum_vec")]
    SumVec { bits: u8, length: u64 },
}
