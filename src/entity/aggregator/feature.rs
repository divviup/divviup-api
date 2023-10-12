use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    TokenHash,
    Unknown(String),
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Features(HashSet<Feature>);

impl Features {
    pub fn token_hash_enabled(&self) -> bool {
        self.0.contains(&Feature::TokenHash)
    }
}

impl From<Feature> for Features {
    fn from(value: Feature) -> Self {
        Self::from_iter([value])
    }
}

impl FromIterator<Feature> for Features {
    fn from_iter<T: IntoIterator<Item = Feature>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
