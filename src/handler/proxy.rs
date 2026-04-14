//! Temporary reverse proxy handler that forwards unmatched Trillium requests to
//! the local Axum server. This exists only during the incremental migration and
//! will be removed once all routes have been moved to Axum.

use reqwest::header::{HeaderName, CONNECTION, HOST, TE, TRAILER, TRANSFER_ENCODING};
use std::net::SocketAddr;
use trillium::{Conn, Handler, Status};

/// A Trillium [`Handler`] that proxies unhalted requests to a local Axum server.
#[derive(Debug)]
pub struct AxumProxy {
    upstream: String,
    client: reqwest::Client,
}

impl AxumProxy {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            upstream: format!("http://[::1]:{}", addr.port()),
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("failed to build proxy HTTP client"),
        }
    }
}

/// Hop-by-hop headers that should not be forwarded through the proxy (RFC 7230 §6.1).
const UNPROXYABLE_HEADERS: [HeaderName; 5] = [HOST, TRANSFER_ENCODING, CONNECTION, TE, TRAILER];

#[trillium::async_trait]
impl Handler for AxumProxy {
    async fn run(&self, mut conn: Conn) -> Conn {
        // Only proxy requests that haven't been handled by earlier handlers.
        if conn.status().is_some() || conn.is_halted() {
            return conn;
        }

        let method = conn.method();
        let path = conn.path();
        let querystring = conn.querystring();

        let url = if querystring.is_empty() {
            format!("{}{}", self.upstream, path)
        } else {
            format!("{}{}?{}", self.upstream, path, querystring)
        };

        let reqwest_method = match reqwest::Method::from_bytes(method.as_ref().as_bytes()) {
            Ok(m) => m,
            Err(_) => return conn.with_status(Status::BadRequest).halt(),
        };

        let mut builder = self.client.request(reqwest_method, &url);

        // Forward request headers, filtering out hop-by-hop headers.
        for (name, values) in conn.request_headers() {
            let header_name = match HeaderName::from_bytes(name.as_ref().as_bytes()) {
                Ok(h) => h,
                Err(_) => continue,
            };
            if UNPROXYABLE_HEADERS.contains(&header_name) {
                continue;
            }
            for value in values.iter() {
                if let Some(s) = value.as_str() {
                    builder = builder.header(&header_name, s);
                }
            }
        }

        // Forward the request body. Note: no size limit is enforced here;
        // the Trillium API layer (trillium-api) enforces a 1 MiB limit before
        // requests reach this handler, so it's fine for the migration window.
        let body = conn.request_body().await.read_bytes().await;
        match body {
            Ok(bytes) if !bytes.is_empty() => {
                builder = builder.body(bytes);
            }
            Err(e) => {
                log::error!("axum proxy error reading request body: {e}");
                return conn.with_status(Status::BadRequest).halt();
            }
            _ => {}
        }

        let resp = match builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("axum proxy error: {e}");
                return conn.with_status(Status::BadGateway).halt();
            }
        };

        let status = resp.status().as_u16();
        let resp_headers = resp.headers().clone();
        let body = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                log::error!("axum proxy error reading response: {e}");
                return conn.with_status(Status::BadGateway).halt();
            }
        };

        let mut conn = conn.with_status(status).halt();

        for (name, value) in resp_headers.iter() {
            if UNPROXYABLE_HEADERS.contains(name) {
                continue;
            }
            if let Ok(v) = value.to_str() {
                conn.response_headers_mut()
                    .append(name.as_str().to_owned(), v.to_owned());
            }
        }

        // This copies the response body; we could avoid it by streaming via
        // resp.into_body() + Body::new_streaming, but it's not worth the extra
        // plumbing for a temporary shim.
        conn.with_body(body.to_vec())
    }
}
