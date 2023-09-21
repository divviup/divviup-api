use crate::entity::task::vdaf::{
    BucketLength, CategoricalBuckets, ContinuousBuckets, CountVec, Histogram, Sum, SumVec, Vdaf,
};

#[test]
fn json_vdaf() {
    for (serialized, vdaf) in [
        (r#"{"type":"count"}"#, Vdaf::Count),
        (
            r#"{"type":"histogram","buckets":["A","B"]}"#,
            Vdaf::Histogram(Histogram::Categorical(CategoricalBuckets {
                buckets: Some(Vec::from(["A".to_owned(), "B".to_owned()])),
                chunk_length: None,
            })),
        ),
        (
            r#"{"type":"histogram","buckets":["A","B"],"chunk_length":2}"#,
            Vdaf::Histogram(Histogram::Categorical(CategoricalBuckets {
                buckets: Some(Vec::from(["A".to_owned(), "B".to_owned()])),
                chunk_length: Some(2),
            })),
        ),
        (
            r#"{"type":"histogram","buckets":[1,10,100]}"#,
            Vdaf::Histogram(Histogram::Continuous(ContinuousBuckets {
                buckets: Some(Vec::from([1, 10, 100])),
                chunk_length: None,
            })),
        ),
        (
            r#"{"type":"histogram","buckets":[1,10,100],"chunk_length":2}"#,
            Vdaf::Histogram(Histogram::Continuous(ContinuousBuckets {
                buckets: Some(Vec::from([1, 10, 100])),
                chunk_length: Some(2),
            })),
        ),
        (
            r#"{"type":"histogram","length":5}"#,
            Vdaf::Histogram(Histogram::Opaque(BucketLength {
                length: 5,
                chunk_length: None,
            })),
        ),
        (
            r#"{"type":"histogram","length":5,"chunk_length":2}"#,
            Vdaf::Histogram(Histogram::Opaque(BucketLength {
                length: 5,
                chunk_length: Some(2),
            })),
        ),
        (
            r#"{"type":"sum","bits":8}"#,
            Vdaf::Sum(Sum { bits: Some(8) }),
        ),
        (
            r#"{"type":"count_vec","length":5}"#,
            Vdaf::CountVec(CountVec {
                length: Some(5),
                chunk_length: None,
            }),
        ),
        (
            r#"{"type":"count_vec","length":5,"chunk_length":2}"#,
            Vdaf::CountVec(CountVec {
                length: Some(5),
                chunk_length: Some(2),
            }),
        ),
        (
            r#"{"type":"sum_vec","bits":8,"length":10}"#,
            Vdaf::SumVec(SumVec {
                bits: Some(8),
                length: Some(10),
                chunk_length: None,
            }),
        ),
        (
            r#"{"type":"sum_vec","bits":8,"length":10,"chunk_length":12}"#,
            Vdaf::SumVec(SumVec {
                bits: Some(8),
                length: Some(10),
                chunk_length: Some(12),
            }),
        ),
        (r#"{"type":"wrong"}"#, Vdaf::Unrecognized),
    ] {
        assert_eq!(serde_json::from_str::<Vdaf>(serialized).unwrap(), vdaf);
    }
}
