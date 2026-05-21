#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

use axum::{
    body::Body,
    extract::Request,
    http::header::{self, HeaderName, HeaderValue},
    middleware::from_fn_with_state,
    response::Response,
    serve, Router,
};
use divviup_api::{
    clients::{aggregator_client::api_types, HttpClient},
    handler::BuiltApp,
    Config, Crypter, Db,
};
use http_body_util::BodyExt;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    error::Error,
    future::Future,
    iter::repeat_with,
    net::{Ipv6Addr, SocketAddr},
    process::Termination,
    sync::Arc,
};
use tokio::{net::TcpListener, runtime};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use tracing::install_test_trace_subscriber;

pub use axum::http::{header as headers, HeaderMap, Method, StatusCode};
pub use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
pub use divviup_api::{
    entity::{self, *},
    queue::{Job, Queue},
    User,
};
pub use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
pub use querystrong::QueryStrong;
pub use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter, Schema,
};
pub use serde_json::{json, Value};
pub use test_harness::test;
pub use time::OffsetDateTime;
pub use url::Url;

pub type TestResult = Result<(), Box<dyn Error>>;

pub trait IntoTestStatus {
    fn into_status(self) -> StatusCode;
}
impl IntoTestStatus for u16 {
    fn into_status(self) -> StatusCode {
        StatusCode::from_u16(self).expect("invalid status code")
    }
}
impl IntoTestStatus for StatusCode {
    fn into_status(self) -> StatusCode {
        self
    }
}

pub mod fixtures;
pub mod tracing;

pub mod client_logs;
pub use client_logs::{ClientLogs, LoggedConn};

mod api_mocks;
pub use api_mocks::ApiMocks;

const POSTMARK_URL: &str = "https://postmark.example";
const AUTH0_URL: &str = "https://auth.example";

pub fn encode_hpke_config(hpke_config: api_types::HpkeConfig) -> String {
    use divviup_api::clients::aggregator_client::api_types::Encode;
    STANDARD.encode(hpke_config.get_encoded().unwrap())
}

async fn set_up_schema_for<T: EntityTrait>(schema: &Schema, db: &Db, t: T) {
    let backend = db.get_database_backend();
    db.execute(backend.build(&schema.create_table_from_entity(t)))
        .await
        .unwrap();
}

pub async fn set_up_schema(db: &Db) {
    let schema = Schema::new(DbBackend::Sqlite);
    set_up_schema_for(&schema, db, Sessions).await;
    set_up_schema_for(&schema, db, Accounts).await;
    set_up_schema_for(&schema, db, Memberships).await;
    set_up_schema_for(&schema, db, Tasks).await;
    set_up_schema_for(&schema, db, queue::Entity).await;
    set_up_schema_for(&schema, db, Aggregators).await;
    set_up_schema_for(&schema, db, ApiTokens).await;
    set_up_schema_for(&schema, db, CollectorCredentials).await;
}

pub async fn config(mock_router: Router) -> Config {
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0u16))
        .await
        .expect("failed to bind mock server");
    let mock_addr = listener.local_addr().unwrap();
    let mock_base = format!("http://[::1]:{}", mock_addr.port());

    tokio::spawn(async move {
        serve(listener, mock_router)
            .await
            .expect("mock server error");
    });

    let reqwest_client = Client::builder()
        .no_proxy()
        .build()
        .expect("failed to build reqwest client");
    let http_client =
        HttpClient::new(reqwest_client).with_proxy_base(mock_base.parse::<url::Url>().unwrap());

    Config {
        session_secrets: repeat_with(|| fastrand::u8(..))
            .take(32)
            .collect::<Vec<_>>()
            .into(),
        api_url: "https://api.example".parse().unwrap(),
        app_url: "https://app.example".parse().unwrap(),
        database_url: "sqlite::memory:".parse().unwrap(),
        auth_url: AUTH0_URL.parse().unwrap(),
        auth_client_id: "client id".into(),
        auth_client_secret: "client secret".into(),
        auth_audience: "aud".into(),
        listen_address: SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)),
        monitoring_listen_address: SocketAddr::from((Ipv6Addr::LOCALHOST, 9464)),
        postmark_token: "-".into(),
        email_address: "test@example.test".parse().unwrap(),
        postmark_url: POSTMARK_URL.parse().unwrap(),
        client: http_client,
        crypter: Crypter::from(Crypter::generate_key()),
        trace_use_test_writer: true,
        trace_force_json_writer: false,
        trace_stackdriver_json_output: false,
        trace_chrome: false,
        tokio_console_enabled: false,
        tokio_console_listen_address: "127.0.0.1:6669".parse().unwrap(),
        metrics_refresh_enabled: true,
        ssrf_validation_enabled: false,
    }
}

#[derive(Debug)]
pub struct DivviupApi {
    router: Router,
    db: Db,
    config: Arc<Config>,
}

impl DivviupApi {
    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn crypter(&self) -> &Crypter {
        &self.config.crypter
    }
}

impl From<&DivviupApi> for Queue {
    fn from(app: &DivviupApi) -> Self {
        Self::new(app.db(), app.config(), CancellationToken::new())
    }
}

impl AsRef<Db> for DivviupApi {
    fn as_ref(&self) -> &Db {
        &self.db
    }
}

pub async fn build_test_app() -> (DivviupApi, ClientLogs) {
    install_test_trace_subscriber();
    let api_mocks = ApiMocks::new();
    let client_logs = api_mocks.client_logs();
    let BuiltApp { router, db, config } =
        divviup_api::build_app(config(api_mocks.into_router()).await).await;
    set_up_schema(&db).await;
    let app = DivviupApi { router, db, config };
    (app, client_logs)
}

/// Build a test app using a custom mock `Router` instead of the default [`ApiMocks`].
/// The provided router is wrapped with client-log middleware so all outbound
/// aggregator API requests are captured in the returned [`ClientLogs`].
pub async fn build_test_app_with_mock(mock: Router) -> (DivviupApi, ClientLogs) {
    install_test_trace_subscriber();
    let client_logs = ClientLogs::default();
    let mock_with_logs = mock.layer(from_fn_with_state(
        client_logs.clone(),
        client_logs::client_logs_middleware,
    ));
    let BuiltApp { router, db, config } =
        divviup_api::build_app(config(mock_with_logs).await).await;
    set_up_schema(&db).await;
    let app = DivviupApi { router, db, config };
    (app, client_logs)
}

#[track_caller]
pub fn set_up<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(DivviupApi) -> Fut,
    Fut: Future<Output = Out>,
    Out: Termination,
{
    block_on(async move {
        let (app, _) = build_test_app().await;
        f(app).await
    })
}

pub fn with_client_logs<F, Fut, Out>(f: F) -> Out
where
    F: FnOnce(DivviupApi, ClientLogs) -> Fut,
    Fut: Future<Output = Out>,
    Out: Termination,
{
    block_on(async move {
        let (app, client_logs) = build_test_app().await;
        f(app, client_logs).await
    })
}

/// Create a single-threaded tokio runtime and run `future` to completion.
/// The `test_harness::test` macro calls harness functions (e.g. `set_up`)
/// from a synchronous context, so this is the async entry point for tests.
pub fn block_on<F: Future<Output = T>, T>(future: F) -> T {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

pub const APP_CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

#[derive(Debug)]
pub struct TestRequest {
    method: Method,
    uri: String,
    headers: HeaderMap,
    body: Vec<u8>,
    user: Option<User>,
}

pub fn get(path: impl Into<String>) -> TestRequest {
    TestRequest::new(Method::GET, path)
}

pub fn post(path: impl Into<String>) -> TestRequest {
    TestRequest::new(Method::POST, path)
}

pub fn put(path: impl Into<String>) -> TestRequest {
    TestRequest::new(Method::PUT, path)
}

pub fn patch(path: impl Into<String>) -> TestRequest {
    TestRequest::new(Method::PATCH, path)
}

pub fn delete(path: impl Into<String>) -> TestRequest {
    TestRequest::new(Method::DELETE, path)
}

impl TestRequest {
    fn new(method: Method, uri: impl Into<String>) -> Self {
        Self {
            method,
            uri: uri.into(),
            headers: HeaderMap::new(),
            body: Vec::new(),
            user: None,
        }
    }

    pub fn with_request_header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: TryInto<HeaderName>,
        N::Error: std::fmt::Debug,
        V: TryInto<HeaderValue>,
        V::Error: std::fmt::Debug,
    {
        let name = name.try_into().expect("invalid header name");
        let value = value.try_into().expect("invalid header value");
        self.headers.insert(name, value);
        self
    }

    pub fn with_request_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into().into_bytes();
        self
    }

    pub fn with_request_json<T: Serialize>(self, t: T) -> Self {
        self.with_request_body(serde_json::to_string(&t).unwrap())
    }

    pub fn with_api_host(self) -> Self {
        self.with_request_header(header::HOST, "api.example")
    }

    pub fn with_app_host(self) -> Self {
        self.with_request_header(header::HOST, "app.example")
    }

    pub fn with_api_headers(self) -> Self {
        (if self.method == Method::GET {
            self
        } else {
            self.with_request_header(header::CONTENT_TYPE, APP_CONTENT_TYPE)
        })
        .with_request_header(header::ACCEPT, APP_CONTENT_TYPE)
        .with_api_host()
    }

    pub fn with_auth_header(self, token: HeaderValue) -> Self {
        self.with_request_header(header::AUTHORIZATION, token)
    }

    pub fn with_user(mut self, user: &User) -> Self {
        self.user = Some(user.clone());
        self
    }

    /// In Part 10, refactor all callers from `.with_state(user)` to `.with_user(&user)`.
    pub fn with_state(self, user: User) -> Self {
        self.with_user(&user)
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub async fn run_async(self, app: &DivviupApi) -> TestResponse {
        let body = if self.body.is_empty() {
            Body::empty()
        } else {
            Body::from(self.body)
        };

        let mut builder = Request::builder().method(self.method).uri(&self.uri);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        let mut request = builder.body(body).expect("failed to build request");
        if let Some(user) = self.user {
            request.extensions_mut().insert(user);
        }
        let response = app
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("oneshot request failed");
        TestResponse::from_response(response).await
    }
}

#[derive(Debug)]
pub struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: bytes::Bytes,
}

impl TestResponse {
    async fn from_response(response: Response) -> Self {
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("failed to read response body")
            .to_bytes();
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn response_headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn response_json<T: DeserializeOwned>(&self) -> T {
        assert_eq!(
            self.headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some(APP_CONTENT_TYPE),
            "expected Content-Type {APP_CONTENT_TYPE}"
        );

        serde_json::from_slice(&self.body).expect("could not deserialize response body")
    }

    pub fn response_body_string(&self) -> Option<String> {
        if self.body.is_empty() {
            None
        } else {
            Some(String::from_utf8(self.body.to_vec()).expect("could not decode body as UTF-8"))
        }
    }

    pub fn header_str(&self, name: impl AsRef<str>) -> Option<&str> {
        self.headers
            .get(name.as_ref())
            .and_then(|v| v.to_str().ok())
    }
}

// These are reimplementations of the Trillium assertion macros atop Axum. In Part 10
// we can decide whether to keep these or refactor the tests, but they're used heavily.

#[macro_export]
macro_rules! assert_ok {
    ($resp:expr) => {{
        let ref __resp = $resp;
        assert_eq!(__resp.status(), ::axum::http::StatusCode::OK);
    }};

    ($resp:expr, $body:expr $(, $header_name:expr => $header_val:expr)*) => {{
        let ref __resp = $resp;
        assert_eq!(__resp.status(), ::axum::http::StatusCode::OK);
        assert_eq!(
            __resp.response_body_string().unwrap_or_default(),
            $body
        );
        $(
            assert_eq!(
                __resp.header_str($header_name),
                Some($header_val),
                concat!("expected header ", stringify!($header_name)),
            );
        )*
    }};
}

#[macro_export]
macro_rules! assert_not_found {
    ($resp:expr) => {{
        let ref __resp = $resp;
        assert_eq!(__resp.status(), ::axum::http::StatusCode::NOT_FOUND);
        assert_eq!(__resp.response_body_string().unwrap_or_default(), "");
    }};
}

#[macro_export]
macro_rules! assert_response {
    ($resp:expr, $status:expr) => {{
        let ref __resp = $resp;
        let expected = $crate::IntoTestStatus::into_status($status);
        assert_eq!(__resp.status(), expected);
    }};

    ($resp:expr, $status:expr, $body:expr $(, $header_name:expr => $header_val:expr)*) => {{
        let ref __resp = $resp;
        let expected = $crate::IntoTestStatus::into_status($status);
        assert_eq!(__resp.status(), expected);
        assert_eq!(
            __resp.response_body_string().unwrap_or_default(),
            $body
        );
        $(
            assert_eq!(
                __resp.header_str($header_name),
                Some($header_val),
                concat!("expected header ", stringify!($header_name)),
            );
        )*
    }};
}

#[macro_export]
macro_rules! assert_status {
    ($resp:expr, $status:expr) => {{
        let ref __resp = $resp;
        let expected = $crate::IntoTestStatus::into_status($status);
        assert_eq!(__resp.status(), expected);
    }};
}

#[macro_export]
macro_rules! assert_body_contains {
    ($resp:expr, $pattern:expr) => {{
        let ref __resp = $resp;
        let body = __resp.response_body_string().unwrap_or_default();
        assert!(
            body.contains($pattern),
            "expected body to contain {:?}, got {:?}",
            $pattern,
            body
        );
    }};
}

#[macro_export]
macro_rules! assert_headers {
    ($resp:expr, $($header_name:expr => $header_val:expr),+ $(,)?) => {{
        let ref __resp = $resp;
        $(
            assert_eq!(
                __resp.header_str($header_name),
                Some($header_val),
                concat!("expected header ", stringify!($header_name)),
            );
        )+
    }};
}

#[async_trait::async_trait]
pub trait Reload: Sized {
    async fn reload(&self, db: &impl ConnectionTrait) -> Result<Option<Self>, DbErr>;
}
macro_rules! impl_reload {
    ($model:ty, $entity:ty) => {
        #[async_trait::async_trait]
        impl Reload for $model {
            async fn reload(&self, db: &impl ConnectionTrait) -> Result<Option<Self>, DbErr> {
                <$entity>::find_by_id(self.id.clone()).one(db).await
            }
        }
    };
}

impl_reload!(Account, Accounts);
impl_reload!(Membership, Memberships);
impl_reload!(Task, Tasks);
impl_reload!(Aggregator, Aggregators);
impl_reload!(ApiToken, ApiTokens);
impl_reload!(CollectorCredential, CollectorCredentials);

#[track_caller]
pub fn assert_same_json_representation<Actual, Expected>(actual: &Actual, expected: &Expected)
where
    Actual: Serialize,
    Expected: Serialize,
{
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}
