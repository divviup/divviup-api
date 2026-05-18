use super::{client_logs::client_logs_middleware, ClientLogs, AUTH0_URL, POSTMARK_URL};
use axum::{middleware, Router};

#[derive(Debug)]
pub struct ApiMocks {
    router: Router,
    client_logs: ClientLogs,
}

impl Default for ApiMocks {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiMocks {
    pub fn new() -> Self {
        let client_logs = ClientLogs::default();

        let inner = divviup_api::api_mocks::ApiMocks::new(POSTMARK_URL, AUTH0_URL);
        let router = inner.into_router().layer(middleware::from_fn_with_state(
            client_logs.clone(),
            client_logs_middleware,
        ));

        Self {
            router,
            client_logs,
        }
    }

    pub fn client_logs(&self) -> ClientLogs {
        self.client_logs.clone()
    }

    pub fn into_router(self) -> Router {
        self.router
    }
}
