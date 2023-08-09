use serde::{Deserialize, Serialize};
use std::{
    collections::{
        btree_set::{IntoIter, Iter},
        BTreeSet as Set,
    },
    convert::Infallible,
    fmt::Display,
    str::FromStr,
};

/// https://www.ietf.org/archive/id/draft-ietf-ppm-dap-05.html#name-queries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum QueryTypeName {
    Reserved,
    TimeInterval,
    FixedSize,
    #[serde(untagged)]
    Other(String),
}

impl AsRef<str> for QueryTypeName {
    fn as_ref(&self) -> &str {
        match self {
            QueryTypeName::Reserved => "Reserved",
            QueryTypeName::TimeInterval => "TimeInterval",
            QueryTypeName::FixedSize => "FixedSize",
            QueryTypeName::Other(o) => o,
        }
    }
}

impl Display for QueryTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for QueryTypeName {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Reserved" => Self::Reserved,
            "TimeInterval" => Self::TimeInterval,
            "FixedSize" => Self::FixedSize,
            other => Self::Other(other.into()),
        })
    }
}

impl From<String> for QueryTypeName {
    fn from(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl From<&str> for QueryTypeName {
    fn from(value: &str) -> Self {
        value.parse().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct QueryTypeNameSet(Set<QueryTypeName>);

impl<I> FromIterator<I> for QueryTypeNameSet
where
    I: Into<QueryTypeName>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl IntoIterator for QueryTypeNameSet {
    type Item = QueryTypeName;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryTypeNameSet {
    type Item = &'a QueryTypeName;

    type IntoIter = Iter<'a, QueryTypeName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for QueryTypeNameSet {
    fn default() -> Self {
        Self::from_iter([QueryTypeName::TimeInterval, QueryTypeName::FixedSize])
    }
}

impl QueryTypeNameSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn intersect(&self, other: &QueryTypeNameSet) -> QueryTypeNameSet {
        self.0.intersection(&other.0).cloned().collect()
    }

    pub fn contains(&self, name: &QueryTypeName) -> bool {
        self.0.contains(name)
    }
}
