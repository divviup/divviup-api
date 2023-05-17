#[track_caller]
pub fn assert_errors(validate: impl validator::Validate, field: &str, codes: &[&str]) {
    assert_eq!(
        validate
            .validate()
            .unwrap_err()
            .field_errors()
            .get(field)
            .map(|c| c.iter().map(|error| &error.code).collect::<Vec<_>>())
            .unwrap_or_default(),
        codes
    );
}
