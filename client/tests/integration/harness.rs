pub use divviup_client::DivviupClient;
use std::{future::Future, net::Ipv6Addr, process::Termination};
use tokio::net::TcpListener;
use url::Url;

pub use std::sync::Arc;
pub use test_support::*;

async fn spawn_test_server(router: axum::Router) -> Url {
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("test server error");
    });
    Url::parse(&format!("http://[::1]:{port}/")).unwrap()
}

pub fn with_configured_client<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(Arc<DivviupApi>, Account, DivviupClient) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Termination,
{
    with_configured_client_and_logs(move |app, account, client, _| async move {
        f(app, account, client).await
    })
}

pub fn with_configured_client_and_logs<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(Arc<DivviupApi>, Account, DivviupClient, ClientLogs) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Termination,
{
    with_client_logs(move |app, _api_logs| async move {
        tracing::install_test_trace_subscriber();
        let client_logs = ClientLogs::default();
        let router = app
            .router()
            .clone()
            .layer(axum::middleware::from_fn_with_state(
                client_logs.clone(),
                client_logs::client_logs_middleware,
            ));
        let base_url = spawn_test_server(router).await;
        let app = Arc::new(app);

        let account = fixtures::account(&app).await;
        let (api_token, token) = ApiToken::build(&account);
        api_token.insert(app.db()).await.unwrap();

        let http_client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("failed to build test HTTP client");
        let client = DivviupClient::new(token, http_client).with_url(base_url);

        f(app, account, client, client_logs).await
    })
}
