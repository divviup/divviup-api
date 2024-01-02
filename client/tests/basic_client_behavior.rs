mod harness;
use divviup_client::{CONTENT_TYPE, USER_AGENT};
use harness::{assert_eq, test, *};

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
        log.url.as_str(),
        &format!(
            "https://api.divviup.org/api/accounts/{}/aggregators",
            account.id
        )
    );

    assert_eq!(
        log.request_headers.get_str(KnownHeaderName::Accept),
        Some(CONTENT_TYPE)
    );

    assert_eq!(
        log.request_headers.get_str(KnownHeaderName::UserAgent),
        Some(USER_AGENT)
    );

    assert!(log
        .request_headers
        .get_str(KnownHeaderName::Authorization)
        .unwrap()
        .starts_with("Bearer "));
}
