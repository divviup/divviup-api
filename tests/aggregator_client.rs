mod harness;
use divviup_api::clients::AggregatorClient;
use harness::{test, *};

#[test(harness = set_up)]
async fn get_task_ids(app: DivviupApi) -> TestResult {
    let client = AggregatorClient::new(app.config());
    let task_ids = client.get_task_ids().await?;
    assert_eq!(task_ids.len(), 25); // two pages of 10 plus a final page of 5
    Ok(())
}

#[test(harness = set_up)]
async fn get_task_metrics(app: DivviupApi) -> TestResult {
    let client = AggregatorClient::new(app.config());
    assert!(client.get_task_metrics("fake-task-id").await.is_ok());
    Ok(())
}

#[test(harness = set_up)]
async fn delete_task(app: DivviupApi) -> TestResult {
    let client = AggregatorClient::new(app.config());
    assert!(client.delete_task("fake-task-id").await.is_ok());
    Ok(())
}
