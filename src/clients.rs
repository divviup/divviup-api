pub mod aggregator_client;
pub mod auth0_client;
pub mod postmark_client;

pub use aggregator_client::AggregatorClient;
pub use auth0_client::Auth0Client;
pub use postmark_client::PostmarkClient;

use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use reqwest::{header::HOST, Method};
use url::Url;

/// Header injected by `HttpClient` when proxy rewriting is active, carrying
/// the original (pre-rewrite) URL. Test infrastructure (`ClientLogs`) can
/// read this to reconstruct the intended URL instead of the proxied one.
pub static ORIGINAL_URL_HEADER: HeaderName = HeaderName::from_static("x-original-url");

#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: Option<Url>,
    default_headers: HeaderMap,
    /// When set, all requests are redirected to this address (scheme + host +
    /// port) while the original host is preserved in the `Host` header. This
    /// lets tests point every client at a single mock server that dispatches
    /// by `Host`.
    proxy_base: Option<Url>,
}

impl HttpClient {
    pub fn new(inner: reqwest::Client) -> Self {
        Self {
            inner,
            base_url: None,
            default_headers: HeaderMap::new(),
            proxy_base: None,
        }
    }

    pub fn with_base(mut self, url: impl Into<Url>) -> Self {
        let mut url = url.into();
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        self.base_url = Some(url);
        self
    }

    pub fn with_default_header(
        mut self,
        name: impl Into<HeaderName>,
        value: impl AsRef<str>,
    ) -> Self {
        self.default_headers.insert(
            name.into(),
            HeaderValue::from_str(value.as_ref()).expect("invalid header value"),
        );
        self
    }

    /// Redirect all outgoing requests to `proxy_url` while preserving the
    /// original host in the `Host` header.
    pub fn with_proxy_base(mut self, proxy_url: Url) -> Self {
        self.proxy_base = Some(proxy_url);
        self
    }

    pub fn build_url(&self, path: &str) -> Result<Url, url::ParseError> {
        match &self.base_url {
            Some(base) => base.join(path),
            None => path.parse(),
        }
    }

    fn resolve_url(&self, path_or_url: &str) -> Url {
        self.build_url(path_or_url)
            .unwrap_or_else(|_| path_or_url.parse().expect("invalid URL"))
    }

    /// Apply proxy rewriting: swap scheme+host+port to the proxy address and
    /// return the original host for the `Host` header.
    fn proxy_rewrite(&self, url: &mut Url) -> Option<String> {
        let proxy = self.proxy_base.as_ref()?;
        let original_host = match url.port() {
            Some(port) => format!("{}:{port}", url.host_str().unwrap_or_default()),
            None => url.host_str().unwrap_or_default().to_string(),
        };
        url.set_scheme(proxy.scheme())
            .expect("proxy_rewrite: failed to set scheme");
        url.set_host(proxy.host_str())
            .expect("proxy_rewrite: failed to set host");
        url.set_port(proxy.port())
            .expect("proxy_rewrite: failed to set port");
        Some(original_host)
    }

    fn build_request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let original_url = self.resolve_url(path);
        let mut url = original_url.clone();
        let builder = if let Some(original_host) = self.proxy_rewrite(&mut url) {
            self.inner
                .request(method, url)
                .header(HOST, original_host)
                .header(&ORIGINAL_URL_HEADER, original_url.as_str())
        } else {
            self.inner.request(method, url)
        };
        builder.headers(self.default_headers.clone())
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.build_request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.build_request(Method::POST, path)
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.build_request(Method::PATCH, path)
    }

    /// Send a GET to an absolute URL, ignoring the base URL but applying
    /// default headers and proxy rewriting.
    pub fn get_url(&self, url: Url) -> reqwest::RequestBuilder {
        let original_url = url.clone();
        let mut url = url;
        let builder = if let Some(original_host) = self.proxy_rewrite(&mut url) {
            self.inner
                .get(url)
                .header(HOST, original_host)
                .header(&ORIGINAL_URL_HEADER, original_url.as_str())
        } else {
            self.inner.get(url)
        };
        builder.headers(self.default_headers.clone())
    }
}

#[derive(Debug)]
pub struct HttpStatusNotSuccess {
    pub method: String,
    pub url: Url,
    pub status: Option<StatusCode>,
    pub body: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("unexpected http status {} {} {:?}: {}", .0.method, .0.url, .0.status, .0.body)]
    HttpStatusNotSuccess(Box<HttpStatusNotSuccess>),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}

/// Extension trait on [`reqwest::Response`] to check for success status codes
/// and convert non-success into [`ClientError`].
#[async_trait::async_trait]
pub trait ResponseExt: Sized {
    async fn success_or_client_error(self) -> Result<reqwest::Response, ClientError>;
}

#[async_trait::async_trait]
impl ResponseExt for reqwest::Response {
    async fn success_or_client_error(self) -> Result<reqwest::Response, ClientError> {
        let status = self.status();
        if status.is_success() {
            Ok(self)
        } else {
            let url = self.url().clone();
            let body = self.text().await.unwrap_or_default();
            Err(ClientError::HttpStatusNotSuccess(Box::new(
                HttpStatusNotSuccess {
                    method: String::new(),
                    url,
                    status: Some(status),
                    body,
                },
            )))
        }
    }
}
