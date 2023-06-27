mod harness;
use harness::{assert_eq, test, *};

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
