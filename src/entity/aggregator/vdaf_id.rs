use crate::json_newtype;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vdaf-06#name-iana-considerations-7
#[derive(Debug, Clone, Copy, Eq)]
pub enum VdafId {
    Prio3Count,
    Prio3Sum,
    Prio3Histogram,
    OtherPrio3(u32),
    Poplar1,
    Other(u32),
}

impl PartialOrd for VdafId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u32::from(*self).partial_cmp(&u32::from(*other))
    }
}

impl Ord for VdafId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u32::from(*self).cmp(&u32::from(*other))
    }
}

impl PartialEq for VdafId {
    fn eq(&self, other: &Self) -> bool {
        u32::from(*self) == u32::from(*other)
    }
}

impl Serialize for VdafId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32((*self).into())
    }
}

impl<'de> Deserialize<'de> for VdafId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VdafIdVisitor;

        impl<'de> serde::de::Visitor<'de> for VdafIdVisitor {
            type Value = VdafId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A vdaf id (u32)")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u32::try_from(value)
                    .map_err(|_| E::custom(format!("u32 out of range: {}", value)))
                    .map(Into::into)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u32::try_from(value)
                    .map_err(|_| E::custom(format!("u32 out of range: {}", value)))
                    .map(Into::into)
            }
        }

        deserializer.deserialize_u64(VdafIdVisitor)
    }
}

impl From<VdafId> for u32 {
    fn from(value: VdafId) -> Self {
        match value {
            VdafId::Prio3Count => 0,
            VdafId::Prio3Sum => 1,
            VdafId::Prio3Histogram => 2,
            VdafId::OtherPrio3(n) => n,
            VdafId::Poplar1 => 4096,
            VdafId::Other(n) => n,
        }
    }
}

impl From<u32> for VdafId {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Prio3Count,
            1 => Self::Prio3Sum,
            2 => Self::Prio3Histogram,
            prio3 @ 3..=4095 => Self::OtherPrio3(prio3),
            4096 => Self::Poplar1,
            other => Self::Other(other),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VdafIdSet(BTreeSet<VdafId>);
impl<I> FromIterator<I> for VdafIdSet
where
    I: Into<VdafId>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl IntoIterator for VdafIdSet {
    type Item = VdafId;

    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a VdafIdSet {
    type Item = &'a VdafId;

    type IntoIter = std::collections::btree_set::Iter<'a, VdafId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for VdafIdSet {
    fn default() -> Self {
        Self::from_iter([VdafId::Prio3Count, VdafId::Prio3Sum, VdafId::Prio3Histogram])
    }
}

json_newtype!(VdafIdSet);

impl VdafIdSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn intersect(&self, other: &VdafIdSet) -> VdafIdSet {
        self.0.intersection(&other.0).copied().collect()
    }
}
