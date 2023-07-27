use crate::json_newtype;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// https://www.ietf.org/archive/id/draft-ietf-ppm-dap-05.html#name-queries
#[derive(Debug, Clone, Copy, Eq)]
pub enum QueryTypeId {
    Reserved,
    TimeInterval,
    FixedSize,
    Other(u8),
}

impl PartialOrd for QueryTypeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u8::from(*self).partial_cmp(&u8::from(*other))
    }
}

impl Ord for QueryTypeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u8::from(*self).cmp(&u8::from(*other))
    }
}

impl PartialEq for QueryTypeId {
    fn eq(&self, other: &Self) -> bool {
        u8::from(*self) == u8::from(*other)
    }
}
impl From<QueryTypeId> for u8 {
    fn from(value: QueryTypeId) -> Self {
        match value {
            QueryTypeId::Reserved => 0,
            QueryTypeId::TimeInterval => 1,
            QueryTypeId::FixedSize => 2,
            QueryTypeId::Other(n) => n,
        }
    }
}
impl From<u8> for QueryTypeId {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Reserved,
            1 => Self::TimeInterval,
            2 => Self::FixedSize,
            other => Self::Other(other),
        }
    }
}

impl Serialize for QueryTypeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8((*self).into())
    }
}

impl<'de> Deserialize<'de> for QueryTypeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct QueryTypeIdVisitor;

        impl<'de> serde::de::Visitor<'de> for QueryTypeIdVisitor {
            type Value = QueryTypeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A query type id (u8)")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u8::try_from(value)
                    .map_err(|_| E::custom(format!("u8 out of range: {}", value)))
                    .map(Into::into)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u8::try_from(value)
                    .map_err(|_| E::custom(format!("u8 out of range: {}", value)))
                    .map(Into::into)
            }
        }

        deserializer.deserialize_u64(QueryTypeIdVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct QueryTypeIdSet(BTreeSet<QueryTypeId>);
impl<I> FromIterator<I> for QueryTypeIdSet
where
    I: Into<QueryTypeId>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl IntoIterator for QueryTypeIdSet {
    type Item = QueryTypeId;

    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryTypeIdSet {
    type Item = &'a QueryTypeId;

    type IntoIter = std::collections::btree_set::Iter<'a, QueryTypeId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for QueryTypeIdSet {
    fn default() -> Self {
        Self::from_iter([QueryTypeId::TimeInterval, QueryTypeId::FixedSize])
    }
}

json_newtype!(QueryTypeIdSet);

impl QueryTypeIdSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn intersect(&self, other: &QueryTypeIdSet) -> QueryTypeIdSet {
        self.0.intersection(&other.0).copied().collect()
    }
}
