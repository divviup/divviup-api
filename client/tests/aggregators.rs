mod harness;
use divviup_client::{DivviupClient, NewAggregator, Role};
use harness::with_configured_client;
use std::sync::Arc;
use test_support::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn aggregator_list(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let aggregators = [
        fixtures::aggregator(&app, Some(&account)).await,
        fixtures::aggregator(&app, Some(&account)).await,
    ];
    let response = client.aggregators(account.id).await?;
    assert_same_json_representation(&aggregators, &response);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_aggregator(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let aggregator = client
        .create_aggregator(
            account.id,
            NewAggregator {
                role: Role::Either,
                dap_url: "https://dap.url".parse().unwrap(),
                api_url: "https://api.url".parse().unwrap(),
                name: "my account name".into(),
                bearer_token: "somebearertoken".into(),
            },
        )
        .await?;
    let from_db = Aggregators::find_by_id(aggregator.id)
        .one(app.db())
        .await?
        .unwrap();
    assert_same_json_representation(&aggregator, &from_db);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn delete_aggregator(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let aggregator = fixtures::aggregator(&app, Some(&account)).await;
    client.delete_aggregator(aggregator.id).await?;
    assert!(Aggregators::find_by_id(aggregator.id)
        .one(app.db())
        .await?
        .unwrap()
        .is_tombstoned());
    Ok(())
}

#[test(harness = with_configured_client)]
async fn rename_aggregator(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let aggregator = fixtures::aggregator(&app, Some(&account)).await;
    let name = fixtures::random_name();
    client.rename_aggregator(aggregator.id, &name).await?;
    assert_eq!(
        Aggregators::find_by_id(aggregator.id)
            .one(app.db())
            .await?
            .unwrap()
            .name,
        name
    );
    Ok(())
}

#[test(harness = with_configured_client)]
async fn rotate_bearer_token(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let aggregator = fixtures::aggregator(&app, Some(&account)).await;
    let new_bearer_token = fixtures::random_name();
    client
        .rotate_aggregator_bearer_token(aggregator.id, &new_bearer_token)
        .await?;
    assert_eq!(
        Aggregators::find_by_id(aggregator.id)
            .one(app.db())
            .await?
            .unwrap()
            .bearer_token,
        new_bearer_token
    );
    Ok(())
}
