use crate::json_newtype;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct Histogram {
    #[validate(required, custom = "strictly_increasing")]
    pub buckets: Option<Vec<u64>>,
}

fn strictly_increasing(buckets: &Vec<u64>) -> Result<(), ValidationError> {
    let mut last_bucket = None;
    for bucket in buckets {
        let bucket = *bucket;
        match last_bucket {
            Some(last_bucket) if last_bucket == bucket => {
                return Err(ValidationError::new("unique"));
            }

            Some(last_bucket) if last_bucket > bucket => {
                return Err(ValidationError::new("sorted"));
            }

            _ => {
                last_bucket = Some(bucket);
            }
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

json_newtype!(Vdaf);

impl Validate for Vdaf {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Vdaf::Count => Ok(()),
            Vdaf::Histogram(h) => h.validate(),
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

    #[test]
    fn validate_histogram() {
        assert!(Histogram {
            buckets: Some(vec![0, 1, 2])
        }
        .validate()
        .is_ok());

        assert!(Histogram {
            buckets: Some(vec![0, 2, 1])
        }
        .validate()
        .is_err());

        assert!(Histogram {
            buckets: Some(vec![0, 0, 2])
        }
        .validate()
        .is_err());
    }
}
