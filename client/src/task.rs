use prio::vdaf::prio3::optimal_chunk_length;
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
    pub batch_time_window_size_seconds: Option<u64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub time_precision_seconds: u32,
    #[deprecated = "Not populated. Will be removed in a future release."]
    pub report_count: u32,
    #[deprecated = "Not populated. Will be removed in a future release."]
    pub aggregate_collection_count: u32,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub expiration: Option<OffsetDateTime>,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
    pub collector_credential_id: Uuid,
    pub report_counter_interval_collected: i64,
    pub report_counter_decode_failure: i64,
    pub report_counter_decrypt_failure: i64,
    pub report_counter_expired: i64,
    pub report_counter_outdated_key: i64,
    pub report_counter_success: i64,
    pub report_counter_too_early: i64,
    pub report_counter_task_expired: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NewTask {
    pub name: String,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
    pub vdaf: Vdaf,
    pub min_batch_size: u64,
    pub max_batch_size: Option<u64>,
    pub batch_time_window_size_seconds: Option<u64>,
    pub time_precision_seconds: u64,
    pub collector_credential_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Vdaf {
    #[serde(rename = "count")]
    Count,

    #[serde(rename = "histogram")]
    Histogram(Histogram),

    #[serde(rename = "sum")]
    Sum { bits: u8 },

    #[serde(rename = "count_vec")]
    CountVec {
        length: u64,
        chunk_length: Option<u64>,
    },

    #[serde(rename = "sum_vec")]
    SumVec(SumVec),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum Histogram {
    Categorical {
        buckets: Vec<String>,
        chunk_length: Option<u64>,
    },
    Continuous {
        buckets: Vec<u64>,
        chunk_length: Option<u64>,
    },
    Length {
        length: u64,
        chunk_length: Option<u64>,
    },
}

impl Histogram {
    /// The length of this histogram, i.e. the number of buckets.
    pub fn length(&self) -> usize {
        match self {
            Histogram::Categorical { buckets, .. } => buckets.len(),
            Histogram::Continuous { buckets, .. } => buckets.len() + 1,
            Histogram::Length { length, .. } => *length as usize,
        }
    }

    /// The chunk length used in the VDAF.
    pub fn chunk_length(&self) -> usize {
        match self {
            Histogram::Categorical { chunk_length, .. } => chunk_length,
            Histogram::Continuous { chunk_length, .. } => chunk_length,
            Histogram::Length { chunk_length, .. } => chunk_length,
        }
        .map(|c| c as usize)
        .unwrap_or_else(|| optimal_chunk_length(self.length()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SumVec {
    pub bits: u8,
    pub length: u64,
    chunk_length: Option<u64>,
}

impl SumVec {
    /// Create a new SumVec
    pub fn new(bits: u8, length: u64, chunk_length: Option<u64>) -> Self {
        Self {
            bits,
            length,
            chunk_length,
        }
    }

    /// The chunk length used in the VDAF.
    pub fn chunk_length(&self) -> usize {
        self.chunk_length
            .map(|c| c as usize)
            .unwrap_or_else(|| optimal_chunk_length(self.bits as usize * self.length as usize))
    }
}
