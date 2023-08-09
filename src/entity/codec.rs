use janus_messages::codec::{Decode, Encode};
use sea_orm::{
    entity::ColumnType,
    sea_query::{ArrayType, BlobSize, Nullable, ValueType, ValueTypeErr},
    ColIdx, DbErr, QueryResult, TryGetError, TryGetable, Value,
};
use serde::{Deserialize, Serialize};
use std::{
    any::type_name,
    fmt::{self, Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize, Default)]
pub struct Codec<T>(pub T);
impl<T: Display> Display for Codec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: PartialEq> PartialEq<T> for Codec<T> {
    fn eq(&self, other: &T) -> bool {
        &self.0 == other
    }
}
impl<T> Deref for Codec<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Codec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Encode + Decode> From<Codec<T>> for Value {
    fn from(value: Codec<T>) -> Self {
        Value::Bytes(Some(Box::new(value.0.get_encoded())))
    }
}

impl<T> From<T> for Codec<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T> Codec<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Encode + Decode> TryGetable for Codec<T> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match res
            .try_get_by::<Vec<u8>, _>(idx)
            .map_err(TryGetError::DbErr)?
        {
            empty if empty.is_empty() => Err(TryGetError::Null(format!("{idx:?}"))),
            bytes => T::get_decoded(&bytes)
                .map(Codec)
                .map_err(|e| TryGetError::DbErr(DbErr::Custom(e.to_string()))),
        }
    }
}

impl<T: Encode + Decode> ValueType for Codec<T> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Bytes(Some(x)) => T::get_decoded(&x).map(Codec).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        type_name::<T>().to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Bytes
    }

    fn column_type() -> ColumnType {
        ColumnType::Binary(BlobSize::Blob(None))
    }
}

impl<T: Encode + Decode> Nullable for Codec<T> {
    fn null() -> Value {
        Value::Bytes(Some(Box::default()))
    }
}
