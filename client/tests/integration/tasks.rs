use crate::harness::{assert_eq, test, *};
use divviup_api::entity::aggregator::Features;
use divviup_client::{NewTask, Vdaf};

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
    let collector_credential = fixtures::collector_credential(&app, &account).await;
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
                collector_credential_id: collector_credential.id,
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
async fn collector_auth_tokens_no_token_hash(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let task = fixtures::task(&app, &account).await;

    let mut leader = task.leader_aggregator(app.db()).await?.into_active_model();
    leader.features = ActiveValue::Set(Features::default().into());
    leader.update(app.db()).await?;

    let tokens = client.task_collector_auth_tokens(&task.id).await?;
    assert!(!tokens.is_empty()); // we don't have aggregator-api client logs here
    Ok(())
}

#[test(harness = with_configured_client)]
async fn collector_auth_tokens_token_hash(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let task = fixtures::task(&app, &account).await;
    let leader = task.leader_aggregator(app.db()).await?;
    assert!(leader.features.token_hash_enabled());
    assert!(client.task_collector_auth_tokens(&task.id).await.is_err());
    Ok(())
}
