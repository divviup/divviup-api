use serde::Deserialize;
use std::{
    fmt::{Display, Formatter, Result},
    sync::{Arc, RwLock},
};
use trillium::{async_trait, Body, Conn, Handler, Headers, Method, StateSet, Status};
use url::Url;

#[derive(Debug, Clone)]
pub struct LoggedConn {
    pub url: Url,
    pub method: Method,
    pub response_body: Option<String>,
    pub response_status: Status,
    pub request_headers: Headers,
    pub response_headers: Headers,
    pub state: Arc<StateSet>,
}

impl LoggedConn {
    pub fn response_json<'a: 'de, 'de, T: Deserialize<'de>>(&'a self) -> T {
        serde_json::from_str(self.response_body.as_ref().unwrap()).expect("deserialization error")
    }
}

impl From<&mut Conn> for LoggedConn {
    fn from(conn: &mut Conn) -> Self {
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

        let state = Arc::new(std::mem::take(conn.inner_mut().state_mut()));

        let request_headers = match state.get() {
            Some(OriginalRequestHeaders(headers)) => headers.clone(),
            None => conn.request_headers().clone(),
        };

        Self {
            url,
            state,
            method: conn.method(),
            response_body: conn
                .inner()
                .response_body()
                .and_then(Body::static_bytes)
                .map(|s| String::from_utf8_lossy(s).to_string()),
            response_status: conn.status().unwrap_or(Status::NotFound),
            request_headers,
            response_headers: conn.response_headers().clone(),
        }
    }
}

impl Display for LoggedConn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "{} {}: {}",
            self.method, self.url, self.response_status
        ))
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClientLogs {
    pub(super) logged_conns: Arc<RwLock<Vec<LoggedConn>>>,
}

impl ClientLogs {
    pub fn len(&self) -> usize {
        self.logged_conns.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.logged_conns.read().unwrap().is_empty()
    }

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
            .filter(|lc| lc.url == url)
            .cloned()
            .collect()
    }
}

#[derive(Debug)]
struct OriginalRequestHeaders(Headers);

#[async_trait]
impl Handler for ClientLogs {
    async fn run(&self, conn: Conn) -> Conn {
        let request_headers = conn.request_headers().clone();
        conn.with_state(OriginalRequestHeaders(request_headers))
    }
    async fn before_send(&self, mut conn: Conn) -> Conn {
        self.logged_conns
            .write()
            .unwrap()
            .push(LoggedConn::from(&mut conn));
        conn
    }
}
