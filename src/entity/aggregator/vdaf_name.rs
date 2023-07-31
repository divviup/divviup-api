use crate::json_newtype;
use serde::{Deserialize, Serialize};
use std::{
    collections::{
        btree_set::{IntoIter, Iter},
        BTreeSet as Set,
    },
    convert::Infallible,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vdaf-06#name-iana-considerations-7
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum VdafName {
    Prio3Count,
    Prio3Sum,
    Prio3Histogram,
    Prio3CountVec,
    Prio3SumVec,
    Poplar1,
    #[serde(untagged)]
    Other(String),
}

impl FromStr for VdafName {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Prio3Count" => Self::Prio3Count,
            "Prio3Sum" => Self::Prio3Sum,
            "Prio3Histogram" => Self::Prio3Histogram,
            "Prio3CountVec" => Self::Prio3CountVec,
            "Prio3SumVec" => Self::Prio3SumVec,
            other => Self::Other(other.into()),
        })
    }
}

impl From<String> for VdafName {
    fn from(value: String) -> Self {
        value.parse().unwrap()
    }
}

impl From<&str> for VdafName {
    fn from(value: &str) -> Self {
        value.parse().unwrap()
    }
}

impl AsRef<str> for VdafName {
    fn as_ref(&self) -> &str {
        match self {
            VdafName::Prio3Count => "Prio3Count",
            VdafName::Prio3Sum => "Prio3Sum",
            VdafName::Prio3Histogram => "Prio3Histogram",
            VdafName::Prio3CountVec => "Prio3CountVec",
            VdafName::Prio3SumVec => "Prio3SumVec",
            VdafName::Poplar1 => "Poplar1",
            VdafName::Other(o) => o,
        }
    }
}

impl Display for VdafName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VdafNameSet(Set<VdafName>);
impl<I> FromIterator<I> for VdafNameSet
where
    I: Into<VdafName>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl IntoIterator for VdafNameSet {
    type Item = VdafName;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a VdafNameSet {
    type Item = &'a VdafName;

    type IntoIter = Iter<'a, VdafName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for VdafNameSet {
    fn default() -> Self {
        Self::from_iter([
            VdafName::Prio3Count,
            VdafName::Prio3Sum,
            VdafName::Prio3Histogram,
        ])
    }
}

json_newtype!(VdafNameSet);

impl VdafNameSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn intersect(&self, other: &VdafNameSet) -> VdafNameSet {
        self.0.intersection(&other.0).cloned().collect()
    }
}
