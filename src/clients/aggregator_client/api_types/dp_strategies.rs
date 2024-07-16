use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "dp_strategy")]
pub enum Prio3Histogram {
    NoDifferentialPrivacy,
    PureDpDiscreteLaplace(PureDpDiscreteLaplace),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "dp_strategy")]
pub enum Prio3SumVec {
    NoDifferentialPrivacy,
    PureDpDiscreteLaplace(PureDpDiscreteLaplace),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PureDpDiscreteLaplace {
    pub budget: PureDpBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PureDpBudget {
    pub epsilon: [Vec<u32>; 2],
}
