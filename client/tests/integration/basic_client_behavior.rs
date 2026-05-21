use crate::harness::{assert_eq, test, *};
use divviup_client::CONTENT_TYPE;

#[test(harness = with_configured_client_and_logs)]
async fn default_headers(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
    logs: ClientLogs,
) {
    let _ = client.aggregators(account.id).await;
    let log = logs.last();
    assert_eq!(
        log.url.path(),
        &format!("/api/accounts/{}/aggregators", account.id)
    );

    assert_eq!(
        log.request_headers
            .get(headers::ACCEPT)
            .and_then(|v| v.to_str().ok()),
        Some(CONTENT_TYPE)
    );

    assert!(log
        .request_headers
        .get(headers::AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("Bearer "));
}
