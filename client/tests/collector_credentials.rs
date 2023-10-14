mod harness;
use divviup_api::api_mocks::aggregator_api::random_hpke_config;
use divviup_client::DivviupClient;
use harness::with_configured_client;
use std::sync::Arc;
use test_support::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn collector_credentials_list(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    fixtures::collector_credential(&app, &account).await;
    let collector_credentials = client.collector_credentials(account.id).await?;
    assert_eq!(collector_credentials.len(), 1);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_collector_credential(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let config = random_hpke_config();
    let collector_credential = client
        .create_collector_credential(account.id, &config, None)
        .await?;
    assert_eq!(config, collector_credential.hpke_config);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_collector_credential_with_name(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let config = random_hpke_config();
    let name = fixtures::random_name();
    let collector_credential = client
        .create_collector_credential(account.id, &config, Some(&name))
        .await?;
    assert_eq!(config, collector_credential.hpke_config);
    assert_eq!(name, collector_credential.name.unwrap());
    Ok(())
}

#[test(harness = with_configured_client)]
async fn delete_collector_credential(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let collector_credential = fixtures::collector_credential(&app, &account).await;
    client
        .delete_collector_credential(collector_credential.id)
        .await?;
    assert!(collector_credential
        .reload(app.db())
        .await?
        .unwrap()
        .is_tombstoned());
    Ok(())
}
