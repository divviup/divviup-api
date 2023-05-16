use serde_json::Value;
use std::{
    fmt::{Display, Formatter, Result},
    sync::{Arc, RwLock},
};
use trillium::{Body, Conn, Headers, Method, Status};
use url::Url;

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
