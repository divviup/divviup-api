use divviup_api::entity::task::vdaf::Vdaf;
use test_support::{assert_eq, test, *};
#[test]
pub fn histogram_representations() {
    let scenarios = [
        (
            json!({"type": "histogram", "buckets": ["a", "b", "c"]}),
            Protocol::Dap04,
            Err(json!({"buckets": [{"code": "must-be-numbers", "message": null, "params": {}}]})),
        ),
        (
            json!({"type": "histogram", "buckets": ["a", "b", "c"]}),
            Protocol::Dap07,
            Ok(json!({"Prio3Histogram": {"length": 3}})),
        ),
        (
            json!({"type": "histogram", "buckets": [1, 2, 3]}),
            Protocol::Dap04,
            Ok(json!({"Prio3Histogram": {"buckets": [1, 2, 3]}})),
        ),
        (
            json!({"type": "histogram", "buckets": [1, 2, 3]}),
            Protocol::Dap07,
            Ok(json!({"Prio3Histogram": {"length": 4}})),
        ),
        (
            json!({"type": "histogram", "length": 3}),
            Protocol::Dap04,
            Err(json!({"buckets": [{"code": "required", "message": null, "params":{}}]})),
        ),
        (
            json!({"type": "histogram", "length": 3}),
            Protocol::Dap07,
            Ok(json!({"Prio3Histogram": {"length": 3}})),
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
