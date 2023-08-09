use core::fmt;
use sea_orm::{
    entity::ColumnType,
    sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr},
    ColIdx, DbErr, QueryResult, TryGetError, TryGetable, Value,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, to_value, Value as JsonValue};
use std::{
    any::type_name,
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize, Default)]
pub struct Json<T>(pub T);
impl<T: Display> Display for Json<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: PartialEq> PartialEq<T> for Json<T> {
    fn eq(&self, other: &T) -> bool {
        &self.0 == other
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Serialize + DeserializeOwned> From<Json<T>> for Value {
    fn from(value: Json<T>) -> Self {
        Value::Json(to_value(&value).ok().map(Box::new))
    }
}

impl<T> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T: Serialize + DeserializeOwned> TryGetable for Json<T> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match res.try_get_by(idx).map_err(TryGetError::DbErr)? {
            JsonValue::Null => Err(TryGetError::Null(format!("{idx:?}"))),
            json => from_value(json).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string()))),
        }
    }
}

impl<T: Serialize + DeserializeOwned> ValueType for Json<T> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Json(Some(x)) => from_value(*x).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        type_name::<T>().to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::Json
    }
}

impl<T: Serialize + DeserializeOwned> Nullable for Json<T> {
    fn null() -> Value {
        Value::Json(Some(Box::new(JsonValue::Null)))
    }
}
