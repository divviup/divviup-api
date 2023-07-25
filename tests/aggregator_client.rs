use divviup_api::api_mocks::aggregator_api;
use test_support::{assert_eq, test, *};
use trillium::Handler;

#[test(harness = set_up)]
async fn get_task_ids(app: DivviupApi) -> TestResult {
    let aggregator = fixtures::aggregator(&app, None).await;
    let client = aggregator.client(app.config().client.clone());
    let task_ids = client.get_task_ids().await?;
    assert_eq!(task_ids.len(), 25); // two pages of 10 plus a final page of 5
    Ok(())
}

#[test(harness = set_up)]
async fn get_task_metrics(app: DivviupApi) -> TestResult {
    let aggregator = fixtures::aggregator(&app, None).await;
    let client = aggregator.client(app.config().client.clone());
    assert!(client.get_task_metrics("fake-task-id").await.is_ok());
    Ok(())
}

#[test(harness = set_up)]
async fn delete_task(app: DivviupApi) -> TestResult {
    let aggregator = fixtures::aggregator(&app, None).await;
    let client = aggregator.client(app.config().client.clone());
    assert!(client.delete_task("fake-task-id").await.is_ok());
    Ok(())
}

mod prefixes {
    use divviup_api::{clients::aggregator_client::TaskMetrics, handler::origin_router};
    use trillium_router::router;

    use super::{assert_eq, test, *};

    fn with_random_prefix<F, Fut>(f: F)
    where
        F: FnOnce(DivviupApi, ClientLogs, Aggregator) -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
    {
        block_on(async move {
            let client_logs = ClientLogs::default();
            let prefix = fixtures::random_name();
            let api_url = Url::parse(&format!(
                "https://api.{}.divviup.org/{prefix}",
                fixtures::random_name()
            ))
            .unwrap();
            let api_mocks = (
                trillium_logger::logger(),
                client_logs.clone(),
                origin_router().with_handler(
                    &api_url.origin().ascii_serialization(),
                    router().all(format!("/{prefix}/*"), aggregator_api::mock()),
                ),
            );
            let mut app = DivviupApi::new(config(api_mocks)).await;
            set_up_schema(app.db()).await;
            let mut aggregator = fixtures::aggregator(&app, None).await.into_active_model();
            aggregator.api_url = ActiveValue::Set(api_url.into());
            let aggregator = aggregator.update(app.db()).await.unwrap();
            let mut info = "testing".into();
            app.init(&mut info).await;
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
        let client = aggregator.client(app.config().client.clone());
        let task_ids = client.get_task_ids().await?;

        assert_eq!(
            client_logs
                .logs()
                .iter()
                .map(|l| l.url.as_ref())
                .collect::<Vec<_>>(),
            vec![
                format!("{}/task_ids", aggregator.api_url),
                format!("{}/task_ids?pagination_token=second", aggregator.api_url),
                format!("{}/task_ids?pagination_token=last", aggregator.api_url)
            ]
        );
        assert_eq!(client_logs.logs().len(), 3);
        assert_eq!(task_ids.len(), 25); // two pages of 10 plus a final page of 5
        Ok(())
    }

    #[test(harness = with_random_prefix)]
    async fn get_task_metrics(
        app: DivviupApi,
        client_logs: ClientLogs,
        aggregator: Aggregator,
    ) -> TestResult {
        let client = aggregator.client(app.config().client.clone());
        let metrics = client.get_task_metrics("fake-task-id").await?;
        assert_eq!(client_logs.last().response_json::<TaskMetrics>(), metrics);
        assert_eq!(client_logs.last().method, Method::Get);
        assert_eq!(
            client_logs.last().url.as_ref(),
            format!("{}/tasks/fake-task-id/metrics", aggregator.api_url)
        );
        assert_eq!(client_logs.logs().len(), 1);
        Ok(())
    }

    #[test(harness = with_random_prefix)]
    async fn delete_task(
        app: DivviupApi,
        client_logs: ClientLogs,
        aggregator: Aggregator,
    ) -> TestResult {
        let client = aggregator.client(app.config().client.clone());
        client.delete_task("fake-task-id").await?;
        assert_eq!(client_logs.last().url.path_segments().unwrap().count(), 3);
        assert_eq!(
            client_logs.last().url.as_ref(),
            format!("{}/tasks/fake-task-id", aggregator.api_url)
        );
        assert_eq!(client_logs.last().method, Method::Delete);
        assert_eq!(client_logs.logs().len(), 1);
        Ok(())
    }
}
