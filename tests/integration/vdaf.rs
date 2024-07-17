use divviup_api::entity::task::vdaf::{BucketLength, CategoricalBuckets, Histogram, Vdaf};
use test_support::{assert_eq, test, *};
#[test]
pub fn histogram_representations() {
    let scenarios = [
        (
            json!({"type": "histogram", "buckets": ["a", "b", "c"], "chunk_length": 1}),
            Protocol::Dap09,
            Ok(json!({"Prio3Histogram": {"length": 3, "chunk_length": 1}})),
        ),
        (
            json!({"type": "histogram", "buckets": [1, 2, 3], "chunk_length": 2}),
            Protocol::Dap09,
            Ok(json!({"Prio3Histogram": {"length": 4, "chunk_length": 2}})),
        ),
        (
            json!({"type": "histogram", "length": 3, "chunk_length": 1}),
            Protocol::Dap09,
            Ok(json!({"Prio3Histogram": {"length": 3, "chunk_length": 1}})),
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
    }))
    .representation_for_protocol(&Protocol::Dap09);
}

#[test]
#[should_panic]
fn histogram_representation_dap_09_no_chunk_length_2() {
    let _ = Vdaf::Histogram(Histogram::Opaque(BucketLength {
        length: 3,
        chunk_length: None,
    }))
    .representation_for_protocol(&Protocol::Dap09);
}
