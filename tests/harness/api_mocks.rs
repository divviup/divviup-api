use super::{fixtures, ClientLogs, LoggedConn, AGGREGATOR_API_URL, AUTH0_URL, POSTMARK_URL};
use divviup_api::{aggregator_api_mock::aggregator_api, clients::auth0_client::Token};
use serde_json::json;
use trillium::{async_trait, Conn, Handler};
use trillium_api::Json;
use trillium_macros::Handler;
use trillium_router::router;

fn postmark_mock() -> impl Handler {
    router().post("/email/withTemplate", Json(json!({})))
}

fn auth0_mock() -> impl Handler {
    router()
        .post(
            "/oauth/token",
            Json(Token {
                access_token: "access token".into(),
                expires_in: 60,
                scope: "".into(),
                token_type: "bearer".into(),
            }),
        )
        .post(
            "/api/v2/users",
            Json(json!({ "user_id": fixtures::random_name() })),
        )
        .post(
            "/api/v2/tickets/password-change",
            Json(json!({
                "ticket": format!("{AUTH0_URL}/password_tickets/{}", fixtures::random_name())
            })),
        )
}

#[derive(Handler, Debug)]
pub struct ApiMocks {
    #[handler]
    handler: Box<dyn Handler>,
    client_logs: ClientLogs,
}

impl ApiMocks {
    pub fn new() -> Self {
        let client_logs = ClientLogs::default();

        Self {
            handler: Box::new((
                client_logs.clone(),
                divviup_api::handler::origin_router()
                    .with_handler(POSTMARK_URL, postmark_mock())
                    .with_handler(AGGREGATOR_API_URL, aggregator_api())
                    .with_handler(AUTH0_URL, auth0_mock()),
            )),
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
