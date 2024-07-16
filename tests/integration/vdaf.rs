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
#[should_panic]
fn histogram_representation_dap_09_no_chunk_length_1() {
    let _ = Vdaf::Histogram(Histogram::Categorical(CategoricalBuckets {
        buckets: Some(Vec::from(["a".to_owned(), "b".to_owned(), "c".to_owned()])),
        chunk_length: None,
        dp_strategy: DpStrategy::default(),
    }))
    .representation_for_protocol(&Protocol::Dap09);
}

#[test]
#[should_panic]
fn histogram_representation_dap_09_no_chunk_length_2() {
    let _ = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: None,
        dp_strategy: DpStrategy::default(),
    }))
    .representation_for_protocol(&Protocol::Dap09);
}
