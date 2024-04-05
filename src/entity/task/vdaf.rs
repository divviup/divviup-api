use crate::{
    clients::aggregator_client::api_types::{AggregatorVdaf, HistogramType},
    entity::{aggregator::VdafName, Protocol},
};
use prio::vdaf::prio3::optimal_chunk_length;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, hash::Hash};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum Histogram {
    Opaque(BucketLength),
    Categorical(CategoricalBuckets),
    Continuous(ContinuousBuckets),
}

impl Histogram {
    pub fn length(&self) -> u64 {
        match self {
            Histogram::Categorical(CategoricalBuckets {
                buckets: Some(buckets),
                ..
            }) => buckets.len() as u64,
            Histogram::Continuous(ContinuousBuckets {
                buckets: Some(buckets),
                ..
            }) => buckets.len() as u64 + 1,
            Histogram::Opaque(BucketLength { length, .. }) => *length,
            _ => 0,
        }
    }

    pub fn chunk_length(&self) -> Option<u64> {
        match self {
            Histogram::Categorical(CategoricalBuckets { chunk_length, .. })
            | Histogram::Continuous(ContinuousBuckets { chunk_length, .. })
            | Histogram::Opaque(BucketLength { chunk_length, .. }) => *chunk_length,
        }
    }

    fn representation_for_protocol(
        &self,
        protocol: &Protocol,
    ) -> Result<AggregatorVdaf, ValidationErrors> {
        match (protocol, self) {
            (Protocol::Dap07 | Protocol::Dap09, histogram) => {
                if let Some(chunk_length) = histogram.chunk_length() {
                    Ok(AggregatorVdaf::Prio3Histogram(HistogramType::Opaque {
                        length: histogram.length(),
                        chunk_length: Some(chunk_length),
                    }))
                } else {
                    panic!("chunk_length was not populated");
                }
            }

            (
                Protocol::Dap04,
                Self::Continuous(ContinuousBuckets {
                    buckets: Some(buckets),
                    chunk_length: None,
                }),
            ) => Ok(AggregatorVdaf::Prio3Histogram(HistogramType::Buckets {
                buckets: buckets.clone(),
                chunk_length: None,
            })),

            (
                Protocol::Dap04,
                Self::Continuous(ContinuousBuckets {
                    buckets: _,
                    chunk_length: Some(_),
                }),
            ) => {
                let mut errors = ValidationErrors::new();
                errors.add("chunk_length", ValidationError::new("not-allowed"));
                Err(errors)
            }

            (Protocol::Dap04, Self::Categorical(_)) => {
                let mut errors = ValidationErrors::new();
                errors.add("buckets", ValidationError::new("must-be-numbers"));
                Err(errors)
            }

            (Protocol::Dap04, _) => {
                let mut errors = ValidationErrors::new();
                errors.add("buckets", ValidationError::new("required"));
                Err(errors)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct ContinuousBuckets {
    #[validate(required, length(min = 1), custom(function = "increasing_and_unique"))]
    pub buckets: Option<Vec<u64>>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct CategoricalBuckets {
    #[validate(required, length(min = 1), custom(function = "unique"))]
    pub buckets: Option<Vec<String>>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq, Copy)]
pub struct BucketLength {
    #[validate(range(min = 1))]
    pub length: u64,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

fn unique<T: Hash + Eq>(buckets: &Vec<T>) -> Result<(), ValidationError> {
    if buckets.len() == buckets.iter().collect::<HashSet<_>>().len() {
        Ok(())
    } else {
        Err(ValidationError::new("unique"))
    }
}

fn increasing(buckets: &Vec<u64>) -> Result<(), ValidationError> {
    let Some(mut last) = buckets.first().copied() else {
        return Ok(());
    };

    for bucket in &buckets[1..] {
        if *bucket >= last {
            last = *bucket;
        } else {
            return Err(ValidationError::new("sorted"));
        }
    }
    Ok(())
}

fn increasing_and_unique(buckets: &Vec<u64>) -> Result<(), ValidationError> {
    // Due to limitations in the Validate derive macro, only one custom validator may be applied
    // to each field. This function thus combines two custom validations into one. Unfortunately,
    // only one error `ValidationError` may be added to the struct-level `ValidationErrors` by a
    // single custom validation, so we must short-circuit if one of the two wrapped validations
    // fails. See https://github.com/Keats/validator/issues/308.
    increasing(buckets)?;
    unique(buckets)
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sum {
    #[validate(required)]
    pub bits: Option<u8>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct CountVec {
    #[validate(required)]
    pub length: Option<u64>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SumVec {
    #[validate(required)]
    pub bits: Option<u8>,

    #[validate(required)]
    pub length: Option<u64>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Vdaf {
    #[serde(rename = "count")]
    Count,

    #[serde(rename = "histogram")]
    Histogram(Histogram),

    #[serde(rename = "sum")]
    Sum(Sum),

    #[serde(rename = "count_vec")]
    CountVec(CountVec),

    #[serde(rename = "sum_vec")]
    SumVec(SumVec),

    #[serde(other)]
    Unrecognized,
}

impl Vdaf {
    pub fn name(&self) -> VdafName {
        match self {
            Vdaf::Count => VdafName::Prio3Count,
            Vdaf::Histogram(_) => VdafName::Prio3Histogram,
            Vdaf::Sum(_) => VdafName::Prio3Sum,
            Vdaf::CountVec(_) => VdafName::Prio3Count,
            Vdaf::SumVec(_) => VdafName::Prio3SumVec,
            Vdaf::Unrecognized => VdafName::Other("unsupported".into()),
        }
    }

    pub fn representation_for_protocol(
        &self,
        protocol: &Protocol,
    ) -> Result<AggregatorVdaf, ValidationErrors> {
        match self {
            Self::Histogram(histogram) => histogram.representation_for_protocol(protocol),
            Self::Count => Ok(AggregatorVdaf::Prio3Count),
            Self::Sum(Sum { bits: Some(bits) }) => Ok(AggregatorVdaf::Prio3Sum { bits: *bits }),
            Self::SumVec(SumVec {
                length: Some(length),
                bits: Some(bits),
                chunk_length,
            }) => Ok(AggregatorVdaf::Prio3SumVec {
                bits: *bits,
                length: *length,
                chunk_length: *chunk_length,
            }),
            Self::CountVec(CountVec {
                length: Some(length),
                chunk_length,
            }) => Ok(AggregatorVdaf::Prio3CountVec {
                length: *length,
                chunk_length: *chunk_length,
            }),
            _ => Err(ValidationErrors::new()),
        }
    }

    pub fn populate_chunk_length(&mut self, protocol: &Protocol) {
        match (self, protocol) {
            // Chunk length was already populated, don't change it.
            (
                Self::Histogram(Histogram::Continuous(ContinuousBuckets {
                    chunk_length: Some(_),
                    ..
                })),
                _,
            )
            | (
                Self::Histogram(Histogram::Opaque(BucketLength {
                    chunk_length: Some(_),
                    ..
                })),
                _,
            )
            | (
                Self::Histogram(Histogram::Categorical(CategoricalBuckets {
                    chunk_length: Some(_),
                    ..
                })),
                _,
            )
            | (
                Self::CountVec(CountVec {
                    chunk_length: Some(_),
                    ..
                }),
                _,
            )
            | (
                Self::SumVec(SumVec {
                    chunk_length: Some(_),
                    ..
                }),
                _,
            ) => {}

            // Select a chunk length if the protocol version needs it and it isn't set yet.
            (Self::Histogram(histogram), Protocol::Dap07 | Protocol::Dap09) => {
                let length = histogram.length();
                match histogram {
                    Histogram::Opaque(BucketLength { chunk_length, .. })
                    | Histogram::Categorical(CategoricalBuckets { chunk_length, .. })
                    | Histogram::Continuous(ContinuousBuckets { chunk_length, .. }) => {
                        *chunk_length = Some(optimal_chunk_length(length as usize) as u64)
                    }
                }
            }

            (
                Self::CountVec(CountVec {
                    length: Some(length),
                    chunk_length: chunk_length @ None,
                }),
                Protocol::Dap07 | Protocol::Dap09,
            ) => *chunk_length = Some(optimal_chunk_length(*length as usize) as u64),

            (
                Self::SumVec(SumVec {
                    bits: Some(bits),
                    length: Some(length),
                    chunk_length: chunk_length @ None,
                }),
                Protocol::Dap07 | Protocol::Dap09,
            ) => {
                *chunk_length = Some(optimal_chunk_length(*bits as usize * *length as usize) as u64)
            }

            // Invalid, missing parameters, do nothing.
            (Self::CountVec(CountVec { length: None, .. }), Protocol::Dap07 | Protocol::Dap09)
            | (Self::SumVec(SumVec { bits: None, .. }), Protocol::Dap07 | Protocol::Dap09)
            | (Self::SumVec(SumVec { length: None, .. }), Protocol::Dap07 | Protocol::Dap09) => {}

            // Chunk length is not applicable, either due to VDAF choice or protocol version.
            (Self::Count, _)
            | (Self::Sum { .. }, _)
            | (Self::Unrecognized, _)
            | (_, Protocol::Dap04) => {}
        }
    }
}

impl Validate for Vdaf {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Vdaf::Count => Ok(()),
            Vdaf::Histogram(Histogram::Continuous(buckets)) => buckets.validate(),
            Vdaf::Histogram(Histogram::Categorical(buckets)) => buckets.validate(),
            Vdaf::Histogram(Histogram::Opaque(length)) => length.validate(),
            Vdaf::Sum(s) => s.validate(),
            Vdaf::SumVec(sv) => sv.validate(),
            Vdaf::CountVec(cv) => cv.validate(),
            Vdaf::Unrecognized => {
                let mut errors = ValidationErrors::new();
                errors.add("type", ValidationError::new("unknown"));
                Err(errors)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::assert_errors;

    mod serde;

    #[test]
    fn validate_continuous_histogram() {
        assert!(ContinuousBuckets {
            buckets: Some(vec![0, 1, 2]),
            chunk_length: None,
        }
        .validate()
        .is_ok());

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 2, 1]),
                chunk_length: None,
            },
            "buckets",
            &["sorted"],
        );

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 0, 2]),
                chunk_length: None,
            },
            "buckets",
            &["unique"],
        );
    }

    #[test]
    fn validate_categorical_histogram() {
        assert!(CategoricalBuckets {
            buckets: Some(vec!["a".into(), "b".into()]),
            chunk_length: None,
        }
        .validate()
        .is_ok());

        assert_errors(
            CategoricalBuckets {
                buckets: Some(vec!["a".into(), "a".into()]),
                chunk_length: None,
            },
            "buckets",
            &["unique"],
        );
    }
}
