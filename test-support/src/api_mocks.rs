use super::{ClientLogs, AUTH0_URL, POSTMARK_URL};
use trillium_macros::Handler;

#[derive(Handler, Debug)]
pub struct ApiMocks {
    #[handler]
    handler: (ClientLogs, divviup_api::api_mocks::ApiMocks),
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

        Self {
            handler: (
                client_logs.clone(),
                divviup_api::api_mocks::ApiMocks::new(POSTMARK_URL, AUTH0_URL),
            ),
            client_logs,
        }
    }
    pub fn client_logs(&self) -> ClientLogs {
        self.client_logs.clone()
    }
}
