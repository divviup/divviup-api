pub use divviup_client::DivviupClient;
use std::{future::Future, sync::Arc};
use test_support::*;
use trillium_client::Client;

pub fn with_http_client<F, Fut>(f: F)
where
    F: FnOnce(Arc<DivviupApi>, Client) -> Fut + Send + 'static,
    Fut: Future<Output = TestResult> + Send + 'static,
{
    set_up(move |app| async move {
        let app = Arc::new(app);
        let http_client = Client::new(trillium_testing::connector(app.clone()));
        f(app, http_client).await
    })
}

pub fn with_configured_client<F, Fut>(f: F)
where
    F: FnOnce(Arc<DivviupApi>, Account, DivviupClient) -> Fut + Send + 'static,
    Fut: Future<Output = TestResult> + Send + 'static,
{
    with_http_client(move |app, http_client| async move {
        let account = fixtures::account(&app).await;
        let (api_token, token) = ApiToken::build(&account);
        api_token.insert(app.db()).await.unwrap();
        let client = DivviupClient::new(token, http_client);
        f(app, account, client).await
    })
}
