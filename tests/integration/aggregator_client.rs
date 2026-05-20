use divviup_api::{
    api_mocks::aggregator_api::{self, BAD_BEARER_TOKEN},
    clients::AggregatorClient,
};
use test_support::{assert_eq, test, *};

#[test(harness = with_client_logs)]
async fn get_task_ids(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let aggregator = fixtures::aggregator(&app, None).await;
    let client = aggregator.client(app.config().client.clone(), app.crypter())?;
    let task_ids = client.get_task_ids().await?;
    assert_eq!(task_ids.len(), 25); // two pages of 10 plus a final page of 5

    let logs = client_logs.logs();
    let expected_token = format!("Bearer {}", aggregator.bearer_token(app.crypter()).unwrap());
    for log in &logs {
        assert_eq!(
            log.request_headers.get(headers::ACCEPT).unwrap(),
            "application/vnd.janus.aggregator+json;version=0.1"
        );
        assert_eq!(
            log.request_headers.get(headers::AUTHORIZATION).unwrap(),
            expected_token.as_str()
        );
    }

    let queries = logs.iter().map(|log| log.url.query()).collect::<Vec<_>>();
    assert_eq!(
        &queries,
        &[
            None,
            Some("pagination_token=second"),
            Some("pagination_token=last")
        ]
    );

    Ok(())
}

#[test(harness = with_client_logs)]
async fn get_task_metrics(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let aggregator = fixtures::aggregator(&app, None).await;
    let client = aggregator.client(app.config().client.clone(), app.crypter())?;
    assert!(client.get_task_upload_metrics("fake-task-id").await.is_ok());

    let log = client_logs.last();
    assert_eq!(
        log.request_headers
            .get(headers::ACCEPT)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/vnd.janus.aggregator+json;version=0.1"
    );
    assert_eq!(
        log.request_headers
            .get(headers::AUTHORIZATION)
            .unwrap()
            .to_str()
            .unwrap(),
        &format!("Bearer {}", aggregator.bearer_token(app.crypter()).unwrap())
    );

    assert_eq!(
        log.url.as_ref(),
        &format!(
            "{}tasks/fake-task-id/metrics/uploads",
            aggregator.api_url.as_ref()
        )
    );

    Ok(())
}

#[test(harness = with_client_logs)]
async fn get_config(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    AggregatorClient::get_config(
        app.config().client.clone(),
        "https://aggregator.api.url".parse().unwrap(),
        "token",
    )
    .await?;
    let log = client_logs.last();
    assert_eq!(
        log.request_headers
            .get(headers::ACCEPT)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/vnd.janus.aggregator+json;version=0.1"
    );
    assert_eq!(
        log.request_headers
            .get(headers::AUTHORIZATION)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer token"
    );

    assert_eq!(log.url.as_ref(), "https://aggregator.api.url/");
    Ok(())
}

#[test(harness = set_up)]
async fn get_config_bad_token(app: DivviupApi) -> TestResult {
    assert!(AggregatorClient::get_config(
        app.config().client.clone(),
        "https://aggregator.api.url".parse().unwrap(),
        BAD_BEARER_TOKEN,
    )
    .await
    .is_err());
    Ok(())
}

mod prefixes {
    use divviup_api::clients::aggregator_client::TaskUploadMetrics;

    use axum::Router;

    use super::{assert_eq, test, *};

    fn with_random_prefix<F, Fut>(f: F)
    where
        F: FnOnce(DivviupApi, ClientLogs, Aggregator) -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
    {
        block_on(async move {
            let prefix = fixtures::random_name();
            let mock = Router::new().nest(&format!("/{prefix}"), aggregator_api::mock());
            let (app, client_logs) = build_test_app_with_mock(mock).await;

            let api_url = Url::parse(&format!(
                "https://api.{}.divviup.org/{prefix}/",
                fixtures::random_name()
            ))
            .unwrap();
            let mut aggregator = fixtures::aggregator(&app, None).await.into_active_model();
            aggregator.encrypted_bearer_token = ActiveValue::Set(
                app.crypter()
                    .encrypt(
                        api_url.as_ref().as_bytes(),
                        fixtures::random_name().as_bytes(),
                    )
                    .unwrap(),
            );
            aggregator.api_url = ActiveValue::Set(api_url.into());
            let aggregator = aggregator.update(app.db()).await.unwrap();
            f(app, client_logs, aggregator).await
        })
        .unwrap()
    }

    #[test(harness = with_random_prefix)]
    async fn get_task_ids(
        app: DivviupApi,
        client_logs: ClientLogs,
        aggregator: Aggregator,
    ) -> TestResult {
        let client = aggregator.client(app.config().client.clone(), app.crypter())?;
        let task_ids = client.get_task_ids().await?;

        assert_eq!(
            client_logs
                .logs()
                .iter()
                .map(|l| l.url.as_ref())
                .collect::<Vec<_>>(),
            vec![
                format!("{}task_ids", aggregator.api_url),
                format!("{}task_ids?pagination_token=second", aggregator.api_url),
                format!("{}task_ids?pagination_token=last", aggregator.api_url)
            ]
        );
        assert_eq!(client_logs.logs().len(), 3);
        assert_eq!(task_ids.len(), 25); // two pages of 10 plus a final page of 5
        Ok(())
    }

    #[test(harness = with_random_prefix)]
    async fn get_task_upload_metrics(
        app: DivviupApi,
        client_logs: ClientLogs,
        aggregator: Aggregator,
    ) -> TestResult {
        let client = aggregator.client(app.config().client.clone(), app.crypter())?;
        let metrics = client.get_task_upload_metrics("fake-task-id").await?;
        assert_eq!(
            client_logs.last().response_json::<TaskUploadMetrics>(),
            metrics
        );
        assert_eq!(client_logs.last().method, Method::GET);
        assert_eq!(
            client_logs.last().url.as_ref(),
            format!("{}tasks/fake-task-id/metrics/uploads", aggregator.api_url)
        );
        assert_eq!(client_logs.logs().len(), 1);
        Ok(())
    }
}
