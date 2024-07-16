use crate::{
    clients::aggregator_client::api_types::{
        dp_strategies::{self, PureDpBudget, PureDpDiscreteLaplace},
        AggregatorVdaf, HistogramType,
    },
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
        _protocol: &Protocol,
    ) -> Result<AggregatorVdaf, ValidationErrors> {
        if let Some(chunk_length) = self.chunk_length() {
            Ok(AggregatorVdaf::Prio3Histogram(HistogramType::Opaque {
                length: self.length(),
                chunk_length: Some(chunk_length),
                dp_strategy: self.dp_strategy().representation_histogram(),
            }))
        } else {
            panic!("chunk_length was not populated");
        }
    }

    pub fn dp_strategy(&self) -> &DpStrategy {
        match self {
            Histogram::Opaque(BucketLength { dp_strategy, .. })
            | Histogram::Categorical(CategoricalBuckets { dp_strategy, .. })
            | Histogram::Continuous(ContinuousBuckets { dp_strategy, .. }) => dp_strategy,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct ContinuousBuckets {
    #[validate(
        required,
        length(min = 1),
        custom(function = "increasing"),
        custom(function = "unique")
    )]
    pub buckets: Option<Vec<u64>>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,

    #[serde(default)]
    #[validate(nested, custom(function = "validate_dp_strategy"))]
    pub dp_strategy: DpStrategy,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct CategoricalBuckets {
    #[validate(required, length(min = 1), custom(function = "unique"))]
    pub buckets: Option<Vec<String>>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,

    #[serde(default)]
    #[validate(nested, custom(function = "validate_dp_strategy"))]
    pub dp_strategy: DpStrategy,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct BucketLength {
    #[validate(range(min = 1))]
    pub length: u64,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,

    #[serde(default)]
    #[validate(nested, custom(function = "validate_dp_strategy"))]
    pub dp_strategy: DpStrategy,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq, Default)]
pub struct DpStrategy {
    #[serde(default)]
    pub dp_strategy: DpStrategyKind,

    #[serde(default)]
    #[validate(nested)]
    pub budget: DpBudget,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum DpStrategyKind {
    #[default]
    NoDifferentialPrivacy,
    PureDpDiscreteLaplace,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq, Default)]
pub struct DpBudget {
    #[validate(length(equal = 2))]
    pub epsilon: Option<Vec<Vec<u32>>>,
}

impl DpStrategy {
    fn representation_histogram(&self) -> dp_strategies::Prio3Histogram {
        match (self.dp_strategy, &self.budget.epsilon) {
            (DpStrategyKind::NoDifferentialPrivacy, None) => {
                dp_strategies::Prio3Histogram::NoDifferentialPrivacy
            }
            (DpStrategyKind::NoDifferentialPrivacy, Some(_))
            | (DpStrategyKind::PureDpDiscreteLaplace, None) => panic!("invalid dp strategy"),
            (DpStrategyKind::PureDpDiscreteLaplace, Some(epsilon)) => {
                dp_strategies::Prio3Histogram::PureDpDiscreteLaplace(PureDpDiscreteLaplace {
                    budget: PureDpBudget {
                        epsilon: epsilon.clone().try_into().expect("invalid epsilon"),
                    },
                })
            }
        }
    }

    fn representation_sumvec(&self) -> dp_strategies::Prio3SumVec {
        match (self.dp_strategy, &self.budget.epsilon) {
            (DpStrategyKind::NoDifferentialPrivacy, None) => {
                dp_strategies::Prio3SumVec::NoDifferentialPrivacy
            }
            (DpStrategyKind::NoDifferentialPrivacy, Some(_))
            | (DpStrategyKind::PureDpDiscreteLaplace, None) => panic!("invalid dp strategy"),
            (DpStrategyKind::PureDpDiscreteLaplace, Some(epsilon)) => {
                dp_strategies::Prio3SumVec::PureDpDiscreteLaplace(PureDpDiscreteLaplace {
                    budget: PureDpBudget {
                        epsilon: epsilon.clone().try_into().expect("invalid epsilon"),
                    },
                })
            }
        }
    }
}

fn validate_dp_strategy(dp_strategy: &DpStrategy) -> Result<(), ValidationError> {
    match (dp_strategy.dp_strategy, &dp_strategy.budget.epsilon) {
        (DpStrategyKind::NoDifferentialPrivacy, None) => Ok(()),
        (DpStrategyKind::NoDifferentialPrivacy, Some(_)) => {
            Err(ValidationError::new("extra_epsilon"))
        }
        (DpStrategyKind::PureDpDiscreteLaplace, None) => {
            Err(ValidationError::new("missing_epsilon"))
        }
        (DpStrategyKind::PureDpDiscreteLaplace, Some(_)) => Ok(()),
    }
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

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct SumVec {
    #[validate(required)]
    pub bits: Option<u8>,

    #[validate(required)]
    pub length: Option<u64>,

    #[validate(range(min = 1))]
    pub chunk_length: Option<u64>,

    #[serde(default)]
    #[validate(nested, custom(function = "validate_dp_strategy"))]
    pub dp_strategy: DpStrategy,
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
                dp_strategy,
            }) => Ok(AggregatorVdaf::Prio3SumVec {
                bits: *bits,
                length: *length,
                chunk_length: *chunk_length,
                dp_strategy: dp_strategy.representation_sumvec(),
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

    pub fn populate_chunk_length(&mut self, _protocol: &Protocol) {
        match self {
            // Chunk length was already populated, don't change it.
            Self::Histogram(Histogram::Continuous(ContinuousBuckets {
                chunk_length: Some(_),
                ..
            }))
            | Self::Histogram(Histogram::Opaque(BucketLength {
                chunk_length: Some(_),
                ..
            }))
            | Self::Histogram(Histogram::Categorical(CategoricalBuckets {
                chunk_length: Some(_),
                ..
            }))
            | Self::CountVec(CountVec {
                chunk_length: Some(_),
                ..
            })
            | Self::SumVec(SumVec {
                chunk_length: Some(_),
                ..
            }) => {}

            // Select a chunk length if it isn't set yet.
            Self::Histogram(histogram) => {
                let length = histogram.length();
                match histogram {
                    Histogram::Opaque(BucketLength { chunk_length, .. })
                    | Histogram::Categorical(CategoricalBuckets { chunk_length, .. })
                    | Histogram::Continuous(ContinuousBuckets { chunk_length, .. }) => {
                        *chunk_length = Some(optimal_chunk_length(length as usize) as u64)
                    }
                }
            }

            Self::CountVec(CountVec {
                length: Some(length),
                chunk_length: chunk_length @ None,
            }) => *chunk_length = Some(optimal_chunk_length(*length as usize) as u64),

            Self::SumVec(SumVec {
                bits: Some(bits),
                length: Some(length),
                chunk_length: chunk_length @ None,
                dp_strategy: _,
            }) => {
                *chunk_length = Some(optimal_chunk_length(*bits as usize * *length as usize) as u64)
            }

            // Invalid, missing parameters, do nothing.
            Self::CountVec(CountVec { length: None, .. })
            | Self::SumVec(SumVec { bits: None, .. })
            | Self::SumVec(SumVec { length: None, .. }) => {}

            // Chunk length is not applicable due to VDAF choice.
            Self::Count | Self::Sum { .. } | Self::Unrecognized => {}
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
            dp_strategy: DpStrategy::default(),
        }
        .validate()
        .is_ok());

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 2, 1]),
                chunk_length: None,
                dp_strategy: DpStrategy::default(),
            },
            "buckets",
            &["sorted"],
        );

        assert_errors(
            ContinuousBuckets {
                buckets: Some(vec![0, 0, 2]),
                chunk_length: None,
                dp_strategy: DpStrategy::default(),
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
            dp_strategy: DpStrategy::default(),
        }
        .validate()
        .is_ok());

        assert_errors(
            CategoricalBuckets {
                buckets: Some(vec!["a".into(), "a".into()]),
                chunk_length: None,
                dp_strategy: DpStrategy::default(),
            },
            "buckets",
            &["unique"],
        );
    }
}
