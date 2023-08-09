mod harness;
use divviup_api::{
    api_mocks::aggregator_api::random_hpke_config, clients::aggregator_client::api_types::Encode,
};
use divviup_client::DivviupClient;
use harness::with_configured_client;
use std::sync::Arc;
use test_support::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn hpke_configs_list(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    fixtures::hpke_config(&app, &account).await;
    let hpke_configs = client.hpke_configs(account.id).await?;
    assert_eq!(hpke_configs.len(), 1);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_hpke_config(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let config = random_hpke_config();
    let hpke_config = client
        .create_hpke_config(account.id, config.get_encoded(), None)
        .await?;
    assert_eq!(config, hpke_config.contents);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_hpke_config_with_name(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let config = random_hpke_config();
    let name = fixtures::random_name();
    let hpke_config = client
        .create_hpke_config(account.id, config.get_encoded(), Some(&name))
        .await?;
    assert_eq!(config, hpke_config.contents);
    assert_eq!(name, hpke_config.name.unwrap());
    Ok(())
}

#[test(harness = with_configured_client)]
async fn delete_hpke_config(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let hpke_config = fixtures::hpke_config(&app, &account).await;
    client.delete_hpke_config(hpke_config.id).await?;
    assert!(hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());
    Ok(())
}
