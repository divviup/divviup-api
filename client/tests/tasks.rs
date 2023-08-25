mod harness;
use divviup_client::{DivviupClient, NewTask, Vdaf};
use harness::with_configured_client;
use std::sync::Arc;
use test_support::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn task_list(app: Arc<DivviupApi>, account: Account, client: DivviupClient) -> TestResult {
    let tasks = [
        fixtures::task(&app, &account).await,
        fixtures::task(&app, &account).await,
    ];
    let response_tasks = client.tasks(account.id).await?;
    assert_same_json_representation(&tasks, &response_tasks);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_task(app: Arc<DivviupApi>, account: Account, client: DivviupClient) -> TestResult {
    let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
    let hpke_config = fixtures::hpke_config(&app, &account).await;
    let response_task = client
        .create_task(
            account.id,
            NewTask {
                name: fixtures::random_name(),
                leader_aggregator_id: leader.id,
                helper_aggregator_id: helper.id,
                vdaf: Vdaf::Count,
                min_batch_size: fastrand::i64(100..).try_into().unwrap(),
                max_batch_size: None,
                time_precision_seconds: fastrand::u64(60..2592000),
                hpke_config_id: hpke_config.id,
            },
        )
        .await?;
    let task_from_db = Tasks::find_by_id(&response_task.id)
        .one(app.db())
        .await?
        .unwrap();
    assert_same_json_representation(&task_from_db, &response_task);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn rename_task(app: Arc<DivviupApi>, account: Account, client: DivviupClient) -> TestResult {
    let task = fixtures::task(&app, &account).await;
    let name = fixtures::random_name();
    let response = client.rename_task(&task.id, &name).await?;
    assert_eq!(&response.name, &name);
    assert_eq!(task.reload(app.db()).await?.unwrap().name, name);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn collector_auth_tokens(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let task = fixtures::task(&app, &account).await;
    let tokens = client.task_collector_auth_tokens(&task.id).await?;
    assert!(!tokens.is_empty()); // we don't have aggregator-api client logs here
    Ok(())
}
