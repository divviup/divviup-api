use divviup_api::entity::task::vdaf::{BucketLength, CategoricalBuckets, Histogram, Vdaf};
use task::vdaf::DpStrategy;
use test_support::{assert_eq, test, *};
#[test]
pub fn histogram_representations() {
    let scenarios = [
        (
            json!({"type": "histogram", "buckets": ["a", "b", "c"], "chunk_length": 1}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3Histogram": {"length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "histogram", "buckets": [1, 2, 3], "chunk_length": 2}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3Histogram": {"length": 4, "chunk_length": 2, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "histogram", "length": 3, "chunk_length": 1}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3Histogram": {"length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "histogram", "length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3Histogram": {"length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "histogram", "length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "PureDpDiscreteLaplace", "budget": {"epsilon": [[1], [1]]}}}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3Histogram": {"length": 3, "chunk_length": 1, "dp_strategy": {"dp_strategy": "PureDpDiscreteLaplace", "budget": {"epsilon": [[1], [1]]}}}}),
            ),
        ),
    ];

    for (input, protocol, output) in scenarios {
        let vdaf: Vdaf = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(
            output,
            vdaf.representation_for_protocol(&protocol)
                .map(|o| serde_json::to_value(o).unwrap())
                .map_err(|e| serde_json::to_value(e).unwrap()),
            "{vdaf:?} {input} {protocol}"
        );
    }
}

#[test]
fn sumvec_representations() {
    let scenarios = [
        (
            json!({"type": "sum_vec", "length": 3, "bits": 1, "chunk_length": 1}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3SumVec": {"length": 3, "bits": 1, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "sum_vec", "length": 3, "bits": 1, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3SumVec": {"length": 3, "bits": 1, "chunk_length": 1, "dp_strategy": {"dp_strategy": "NoDifferentialPrivacy"}}}),
            ),
        ),
        (
            json!({"type": "sum_vec", "length": 3, "bits": 1, "chunk_length": 1, "dp_strategy": {"dp_strategy": "PureDpDiscreteLaplace", "budget": {"epsilon": [[1], [1]]}}}),
            Protocol::Dap09,
            Ok(
                json!({"Prio3SumVec": {"length": 3, "bits": 1, "chunk_length": 1, "dp_strategy": {"dp_strategy": "PureDpDiscreteLaplace", "budget": {"epsilon": [[1], [1]]}}}}),
            ),
        ),
    ];

    for (input, protocol, output) in scenarios {
        let vdaf: Vdaf = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(
            output,
            vdaf.representation_for_protocol(&protocol)
                .map(|o| serde_json::to_value(o).unwrap())
                .map_err(|e| serde_json::to_value(e).unwrap()),
            "{vdaf:?} {input} {protocol}"
        );
    }
}

#[test]
fn histogram_representation_dap_09_no_chunk_length_1() {
    let result = Vdaf::Histogram(Histogram::Categorical(CategoricalBuckets {
        buckets: Some(Vec::from(["a".to_owned(), "b".to_owned(), "c".to_owned()])),
        chunk_length: None,
        dp_strategy: DpStrategy::default(),
    }))
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());
}

#[test]
fn histogram_representation_dap_09_no_chunk_length_2() {
    let result = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: None,
        dp_strategy: DpStrategy::default(),
    }))
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());
}

#[test]
fn histogram_invalid_dp_strategy_returns_error() {
    use task::vdaf::{DpBudget, DpStrategyKind};

    // NoDifferentialPrivacy with epsilon present
    let result = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: Some(1),
        dp_strategy: DpStrategy {
            dp_strategy: DpStrategyKind::NoDifferentialPrivacy,
            budget: DpBudget {
                epsilon: Some(vec![vec![1], vec![1]]),
            },
        },
    }))
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());

    // PureDpDiscreteLaplace without epsilon
    let result = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: Some(1),
        dp_strategy: DpStrategy {
            dp_strategy: DpStrategyKind::PureDpDiscreteLaplace,
            budget: DpBudget { epsilon: None },
        },
    }))
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());

    // PureDpDiscreteLaplace with wrong-length epsilon
    let result = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: Some(1),
        dp_strategy: DpStrategy {
            dp_strategy: DpStrategyKind::PureDpDiscreteLaplace,
            budget: DpBudget {
                epsilon: Some(vec![vec![1], vec![2], vec![3]]),
            },
        },
    }))
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());
}

#[test]
fn sumvec_invalid_dp_strategy_returns_error() {
    use task::vdaf::{DpBudget, DpStrategyKind, SumVec};

    // NoDifferentialPrivacy with epsilon present
    let result = Vdaf::SumVec(SumVec {
        bits: Some(1),
        length: Some(3),
        chunk_length: Some(1),
        dp_strategy: DpStrategy {
            dp_strategy: DpStrategyKind::NoDifferentialPrivacy,
            budget: DpBudget {
                epsilon: Some(vec![vec![1], vec![1]]),
            },
        },
    })
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());

    // PureDpDiscreteLaplace without epsilon
    let result = Vdaf::SumVec(SumVec {
        bits: Some(1),
        length: Some(3),
        chunk_length: Some(1),
        dp_strategy: DpStrategy {
            dp_strategy: DpStrategyKind::PureDpDiscreteLaplace,
            budget: DpBudget { epsilon: None },
        },
    })
    .representation_for_protocol(&Protocol::Dap09);
    assert!(result.is_err());
}
