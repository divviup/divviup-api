use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    TokenHash,
    UploadMetrics,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Features(HashSet<Feature>);

impl Features {
    pub fn token_hash_enabled(&self) -> bool {
        self.0.contains(&Feature::TokenHash)
    }

    pub fn upload_metrics_enabled(&self) -> bool {
        self.0.contains(&Feature::UploadMetrics)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn intersect(&self, other: &Features) -> Self {
        self.0.intersection(&other.0).cloned().collect()
    }

    pub fn contains(&self, feature: &Feature) -> bool {
        self.0.contains(feature)
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
    fn features() {
        assert_eq!(
            serde_json::from_value::<Features>(json!(["TokenHash"])).unwrap(),
            Features::from_iter([Feature::TokenHash])
        );
        assert_eq!(
            serde_json::from_value::<Features>(json!(["TokenHash", "TokenHash", "TokenHash"]))
                .unwrap(),
            Features::from_iter([Feature::TokenHash])
        );
        assert_eq!(
            serde_json::from_value::<Features>(json!(["TokenHash", "UploadMetrics"])).unwrap(),
            Features::from_iter([Feature::TokenHash, Feature::UploadMetrics])
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
