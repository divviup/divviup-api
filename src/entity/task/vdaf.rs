use crate::{
    clients::aggregator_client::api_types::{AggregatorVdaf, HistogramType},
    entity::{aggregator::VdafName, Protocol},
};
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
            }) => buckets.len() as u64,
            Histogram::Continuous(ContinuousBuckets {
                buckets: Some(buckets),
            }) => buckets.len() as u64,
            Histogram::Opaque(BucketLength { length }) => *length,
            _ => 0,
        }
    }

    fn representation_for_protocol(
        &self,
        protocol: &Protocol,
    ) -> Result<AggregatorVdaf, ValidationErrors> {
        match (protocol, self) {
            (Protocol::Dap05, histogram) => {
                Ok(AggregatorVdaf::Prio3Histogram(HistogramType::Opaque {
                    length: histogram.length(),
                }))
            }

            (
                Protocol::Dap04,
                Self::Continuous(ContinuousBuckets {
                    buckets: Some(buckets),
                }),
            ) => Ok(AggregatorVdaf::Prio3Histogram(HistogramType::Buckets {
                buckets: buckets.clone(),
            })),

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
    #[validate(required, length(min = 1), custom = "increasing", custom = "unique")]
    pub buckets: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct CategoricalBuckets {
    #[validate(required, length(min = 1), custom = "unique")]
    pub buckets: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq, Copy)]
pub struct BucketLength {
    #[validate(range(min = 1))]
    pub length: u64,
}

fn unique<T: Hash + Eq>(buckets: &[T]) -> Result<(), ValidationError> {
    if buckets.len() == buckets.iter().collect::<HashSet<_>>().len() {
        Ok(())
    } else {
        Err(ValidationError::new("unique"))
    }
}

fn increasing(buckets: &[u64]) -> Result<(), ValidationError> {
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

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sum {
    #[validate(required)]
    pub bits: Option<u8>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct CountVec {
    #[validate(required)]
    pub length: Option<u64>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SumVec {
    #[validate(required)]
    pub bits: Option<u8>,

    #[validate(required)]
    pub length: Option<u64>,
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
            }) => Ok(AggregatorVdaf::Prio3SumVec {
                bits: *bits,
                length: *length,
            }),
            Self::CountVec(CountVec {
                length: Some(length),
            }) => Ok(AggregatorVdaf::Prio3CountVec { length: *length }),
            _ => Err(ValidationErrors::new()),
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

    #[test]
    fn validate_continuous_histogram() {
        assert!(ContinuousBuckets {
            buckets: Some(vec![0, 1, 2])
        }
        .validate()
        .is_ok());

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 2, 1]),
            },
            "buckets",
            &["sorted"],
        );

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 0, 2]),
            },
            "buckets",
            &["unique"],
        );
    }

    #[test]
    fn validate_categorical_histogram() {
        assert!(CategoricalBuckets {
            buckets: Some(vec!["a".into(), "b".into()])
        }
        .validate()
        .is_ok());

        assert_errors(
            CategoricalBuckets {
                buckets: Some(vec!["a".into(), "a".into()]),
            },
            "buckets",
            &["unique"],
        );
    }
}
