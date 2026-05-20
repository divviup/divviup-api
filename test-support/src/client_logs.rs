use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use divviup_api::clients::ORIGINAL_URL_HEADER;
use http_body_util::BodyExt;
use serde::Deserialize;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::{Arc, RwLock},
};
use url::Url;

#[derive(Debug, Clone)]
pub struct LoggedConn {
    pub url: Url,
    pub method: Method,
    pub request_headers: HeaderMap,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub response_status: StatusCode,
    pub response_headers: HeaderMap,
}

impl LoggedConn {
    pub fn response_json<'a: 'de, 'de, T: Deserialize<'de>>(&'a self) -> T {
        serde_json::from_str(self.response_body.as_ref().unwrap()).expect("deserialization error")
    }

    pub fn request_json<T: serde::de::DeserializeOwned>(&self) -> T {
        serde_json::from_str(self.request_body.as_ref().unwrap()).expect("deserialization error")
    }
}

impl Display for LoggedConn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}: {}", self.method, self.url, self.response_status)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClientLogs {
    logged_conns: Arc<RwLock<Vec<LoggedConn>>>,
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

fn reconstruct_url(req: &Request) -> Url {
    if let Some(original) = req
        .headers()
        .get(ORIGINAL_URL_HEADER.as_str())
        .and_then(|v| v.to_str().ok())
    {
        if let Ok(url) = Url::parse(original) {
            return url;
        }
    }

    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let mut url = Url::parse(&format!("https://{host}"))
        .unwrap_or_else(|e| panic!("reconstruct_url: malformed Host: {host}: {e}"));
    url.set_path(req.uri().path());
    url.set_query(req.uri().query());
    url
}

pub async fn client_logs_middleware(
    State(logs): State<ClientLogs>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let request_headers = request.headers().clone();
    let url = reconstruct_url(&request);

    let (parts, body) = request.into_parts();
    let body_bytes = body
        .collect()
        .await
        .expect("failed to read request body")
        .to_bytes();
    let request_body = if body_bytes.is_empty() {
        None
    } else {
        String::from_utf8(body_bytes.to_vec()).ok()
    };
    let request = Request::from_parts(parts, Body::from(body_bytes));

    let response = next.run(request).await;

    let (parts, body) = response.into_parts();
    let response_bytes = body
        .collect()
        .await
        .expect("failed to read response body")
        .to_bytes();
    let response_body = if response_bytes.is_empty() {
        None
    } else {
        String::from_utf8(response_bytes.to_vec()).ok()
    };

    logs.logged_conns.write().unwrap().push(LoggedConn {
        url,
        method,
        request_headers,
        request_body,
        response_body,
        response_status: parts.status,
        response_headers: parts.headers.clone(),
    });

    Response::from_parts(parts, Body::from(response_bytes))
}
