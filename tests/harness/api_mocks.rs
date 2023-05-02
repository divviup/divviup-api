use super::{fixtures, AGGREGATOR_URL, AUTH0_URL, POSTMARK_URL};
use divviup_api::{aggregator_api_mock::aggregator_api, clients::auth0_client::Token};
use serde_json::{json, Value};
use std::sync::Arc;
use trillium::{async_trait, Conn, Handler};
use trillium_api::Json;
use trillium_http::{Body, Headers, Method, Status};
use trillium_macros::Handler;
use trillium_router::router;
use url::Url;

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

#[derive(Debug, Clone)]
pub struct LoggedConn {
    pub url: Url,
    pub method: Method,
    pub response_body: Option<String>,
    pub response_status: Status,
    pub request_headers: Headers,
    pub response_headers: Headers,
}

impl LoggedConn {
    pub fn response_json(&self) -> Value {
        serde_json::from_str(&self.response_body.as_ref().unwrap()).unwrap()
    }
}

impl From<&Conn> for LoggedConn {
    fn from(conn: &Conn) -> Self {
        let url = Url::parse(&format!(
            "{}://{}{}{}",
            if conn.is_secure() { "https" } else { "http" },
            conn.inner().host().unwrap(),
            conn.path(),
            match conn.querystring() {
                "" => "".into(),
                q => format!("?{q}"),
            }
        ))
        .unwrap();

        Self {
            url,
            method: conn.method(),
            response_body: conn
                .inner()
                .response_body()
                .and_then(Body::static_bytes)
                .map(|s| String::from_utf8_lossy(s).to_string()),
            response_status: conn.status().unwrap_or(Status::NotFound),
            request_headers: conn.request_headers().clone(),
            response_headers: conn.response_headers().clone(),
        }
    }
}

impl std::fmt::Display for LoggedConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}: {}",
            self.method, self.url, self.response_status
        ))
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClientLogs {
    logged_conns: Arc<std::sync::RwLock<Vec<LoggedConn>>>,
}

impl ClientLogs {
    pub fn logs(&self) -> Vec<LoggedConn> {
        self.logged_conns.read().unwrap().clone()
    }

    pub fn last(&self) -> LoggedConn {
        self.logged_conns.read().unwrap().last().unwrap().clone()
    }

    pub fn matching_url(&self, url: Url) -> Vec<LoggedConn> {
        self.logged_conns
            .read()
            .unwrap()
            .iter()
            .filter(|lc| (**lc).url == url)
            .cloned()
            .collect()
    }
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
                    .with_handler(AGGREGATOR_URL, aggregator_api())
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
