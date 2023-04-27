use sea_orm::{
    entity::prelude::*,
    sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr},
    TryGetError, TryGetable, Value,
};
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

impl From<Vdaf> for Value {
    fn from(value: Vdaf) -> Self {
        Value::Json(serde_json::to_value(&value).ok().map(Box::new))
    }
}

impl TryGetable for Vdaf {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let json = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
        serde_json::from_value(json).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
    }
}

impl ValueType for Vdaf {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Json(Some(x)) => serde_json::from_value(*x).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Vdaf).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::Json
    }
}

impl Nullable for Vdaf {
    fn null() -> Value {
        Value::Json(Some(Box::new(Json::Null)))
    }
}

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
