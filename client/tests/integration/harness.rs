pub use divviup_client::DivviupClient;
use test_support::tracing::install_test_trace_subscriber;
use trillium_testing::connector;

pub use std::sync::Arc;
pub use test_support::*;

use std::{future::Future, process::Termination};
use trillium_client::Client;

pub fn with_http_client<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(Arc<DivviupApi>, Client, ClientLogs) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Termination,
{
    with_client_logs(move |app, _| async move {
        install_test_trace_subscriber();
        let client_logs = ClientLogs::default();
        let app = Arc::new(app);
        let http_client = Client::new(connector((client_logs.clone(), app.clone())));
        f(app, http_client, client_logs).await
    })
}

pub fn with_configured_client<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(Arc<DivviupApi>, Account, DivviupClient) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Termination,
{
    with_configured_client_and_logs(move |app, account, client, _| async move {
        install_test_trace_subscriber();
        f(app, account, client).await
    })
}

pub fn with_configured_client_and_logs<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(Arc<DivviupApi>, Account, DivviupClient, ClientLogs) -> Fut + Send + 'static,
    Fut: Future<Output = Out> + Send + 'static,
    Out: Termination,
{
    with_http_client(move |app, http_client, logs| async move {
        install_test_trace_subscriber();
        let account = fixtures::account(&app).await;
        let (api_token, token) = ApiToken::build(&account);
        api_token.insert(app.db()).await.unwrap();
        let client = DivviupClient::new(token, http_client);
        f(app, account, client, logs).await
    })
}
