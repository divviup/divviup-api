use num_bigint::BigUint;
use num_rational::Ratio;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
#[serde(tag = "dp_strategy")]
pub enum Prio3Histogram {
    #[default]
    NoDifferentialPrivacy,
    PureDpDiscreteLaplace(PureDpDiscreteLaplace),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
#[serde(tag = "dp_strategy")]
pub enum Prio3SumVec {
    #[default]
    NoDifferentialPrivacy,
    PureDpDiscreteLaplace(PureDpDiscreteLaplace),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct PureDpDiscreteLaplace {
    pub budget: PureDpBudget,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct PureDpBudget {
    pub epsilon: Ratio<BigUint>,
}
