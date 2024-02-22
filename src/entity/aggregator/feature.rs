use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    TokenHash,
    #[serde(untagged)]
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::entity::aggregator::{Feature, Features};

    #[test]
    fn features_none() {
        assert_eq!(
            serde_json::from_value::<Features>(json!([])).unwrap(),
            Features::from_iter([])
        )
    }

    #[test]
    fn features_token_hash() {
        assert_eq!(
            serde_json::from_value::<Features>(json!(["TokenHash"])).unwrap(),
            Features::from_iter([Feature::TokenHash])
        );
        assert_eq!(
            serde_json::from_value::<Features>(json!(["TokenHash", "TokenHash", "TokenHash"]))
                .unwrap(),
            Features::from_iter([Feature::TokenHash])
        );
    }

    #[test]
    fn features_unknown() {
        assert_eq!(
            serde_json::from_value::<Features>(json!(["UnspecifiedUnknownFeature"])).unwrap(),
            Features::from_iter([Feature::Unknown("UnspecifiedUnknownFeature".to_string())])
        );

        assert_eq!(
            serde_json::from_value::<Features>(json!([
                "UnspecifiedUnknownFeature",
                "UnspecifiedUnknownFeature",
                "UnspecifiedUnknownFeature"
            ]))
            .unwrap(),
            Features::from_iter([Feature::Unknown("UnspecifiedUnknownFeature".to_string())])
        );

        assert_eq!(
            serde_json::from_value::<Features>(json!([
                "UnspecifiedUnknownFeature",
                "AnotherUnspecifiedUnknownFeature"
            ]))
            .unwrap(),
            Features::from_iter([
                Feature::Unknown("UnspecifiedUnknownFeature".to_string()),
                Feature::Unknown("AnotherUnspecifiedUnknownFeature".to_string())
            ])
        );

        assert_eq!(
            serde_json::from_value::<Features>(json!(["UnspecifiedUnknownFeature", "TokenHash"]))
                .unwrap(),
            Features::from_iter([
                Feature::TokenHash,
                Feature::Unknown("UnspecifiedUnknownFeature".to_string())
            ])
        );
    }
}
