use super::{ClientLogs, LoggedConn, AUTH0_URL, POSTMARK_URL};
use trillium::{async_trait, Conn, Handler};
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

#[async_trait]
impl Handler for ClientLogs {
    async fn run(&self, conn: Conn) -> Conn {
        conn
    }
    async fn before_send(&self, conn: Conn) -> Conn {
        self.logged_conns
            .write()
            .unwrap()
            .push(LoggedConn::from(&conn));
        conn
    }
}
