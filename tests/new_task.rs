mod harness;
use harness::{test, *};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::random;
use std::iter::repeat_with;

pub async fn assert_errors(app: &DivviupApi, new_task: NewTask, field: &str, codes: &[&str]) {
    let account = fixtures::account(app).await;
    assert_eq!(
        new_task
            .validate(account, app.db())
            .await
            .unwrap_err()
            .field_errors()
            .get(field)
            .map(|c| c.iter().map(|error| &error.code).collect::<Vec<_>>())
            .unwrap_or_default(),
        codes
    );
}

#[test(harness = set_up)]
async fn validation(app: DivviupApi) -> TestResult {
    assert_errors(
        &app,
        NewTask {
            id: Some("tooshort".into()),
            ..Default::default()
        },
        "id",
        &["length"],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            id: Some("🦀".into()),
            ..Default::default()
        },
        "id",
        &["length", "base64"],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            id: Some("\u{205f}".into()),
            ..Default::default()
        },
        "id",
        &["length", "base64"],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            id: Some(std::iter::repeat(' ').take(43).collect()),
            ..Default::default()
        },
        "id",
        &["base64"],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            id: Some(
                URL_SAFE_NO_PAD.encode(repeat_with(random::<u8>).take(32).collect::<Vec<_>>()),
            ),
            ..Default::default()
        },
        "id",
        &[],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            vdaf_verify_key: Some(
                URL_SAFE_NO_PAD.encode(repeat_with(random::<u8>).take(16).collect::<Vec<_>>()),
            ),
            ..Default::default()
        },
        "vdaf_verify_key",
        &[],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            min_batch_size: Some(100),
            max_batch_size: Some(50),
            ..Default::default()
        },
        "min_batch_size",
        &["min_greater_than_max"],
    )
    .await;

    assert_errors(
        &app,
        NewTask {
            min_batch_size: Some(100),
            max_batch_size: Some(50),
            ..Default::default()
        },
        "max_batch_size",
        &["min_greater_than_max"],
    )
    .await;
    Ok(())
}
