use crate::json_newtype;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Eq, PartialEq)]
pub struct Histogram {
    #[validate(required, custom = "sorted", custom = "unique")]
    pub buckets: Option<Vec<i32>>,
}

fn sorted(buckets: &Vec<i32>) -> Result<(), ValidationError> {
    let mut buckets_cloned = buckets.clone();
    buckets_cloned.sort_unstable();
    if &buckets_cloned == buckets {
        Ok(())
    } else {
        Err(ValidationError::new("sorted"))
    }
}

fn unique(buckets: &Vec<i32>) -> Result<(), ValidationError> {
    if buckets.iter().collect::<HashSet<_>>().len() == buckets.len() {
        Ok(())
    } else {
        Err(ValidationError::new("unique"))
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sum {
    #[validate(required)]
    pub bits: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Vdaf {
    #[serde(rename = "count")]
    Count,

    #[serde(rename = "histogram")]
    Histogram(Histogram),

    #[serde(rename = "sum")]
    Sum(Sum), // 128 is ceiling

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
            Vdaf::Unrecognized => {
                let mut errors = ValidationErrors::new();
                errors.add("type", ValidationError::new("unknown"));
                Err(errors)
            }
        }
    }
}
