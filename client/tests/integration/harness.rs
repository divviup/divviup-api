pub use divviup_client::DivviupClient;
use std::{future::Future, net::Ipv6Addr, process::Termination};
use test_support::tracing::install_test_trace_subscriber;
use tokio::net::TcpListener;
use trillium_http::Stopper;
use url::Url;

pub use std::sync::Arc;
pub use test_support::*;

async fn spawn_test_server(app: impl trillium::Handler) -> (Url, Stopper) {
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let stopper = Stopper::new();

    tokio::spawn(
        trillium_tokio::config()
            .without_signals()
            .with_stopper(stopper.clone())
            .with_prebound_server(listener)
            .run_async(app),
    );

    let url = Url::parse(&format!("http://[::1]:{port}/")).unwrap();
    (url, stopper)
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
        install_test_trace_subscriber();
        let client_logs = ClientLogs::default();
        let app = Arc::new(app);

        let (base_url, _stopper) = spawn_test_server((client_logs.clone(), app.clone())).await;

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
