#![allow(dead_code)]
// because different tests use different parts of this
use base64::{engine::general_purpose::STANDARD, Engine};
use divviup_api::{
    clients::aggregator_client::api_types::{Encode, HpkeConfig},
    entity::queue,
    ApiConfig, Db,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, future::Future};
use trillium::Handler;
use trillium_client::Client;
use trillium_testing::TestConn;

pub use divviup_api::{
    entity::{self, *},
    queue::{Job, Queue},
    DivviupApi, User,
};
pub use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
pub use querystrong::QueryStrong;
pub use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, Schema, Set,
};
pub use serde_json::{json, Value};
pub use test_harness::test;
pub use trillium::{Conn, KnownHeaderName, Method, Status};
pub use trillium_testing::prelude::*;
pub use url::Url;

pub type TestResult = Result<(), Box<dyn Error>>;

pub mod fixtures;

mod client_logs;
pub use client_logs::{ClientLogs, LoggedConn};

mod api_mocks;
pub use api_mocks::ApiMocks;

const POSTMARK_URL: &str = "https://postmark.example";
const AUTH0_URL: &str = "https://auth.example";

pub fn encode_hpke_config(hpke_config: HpkeConfig) -> String {
    let mut vec = Vec::new();
    hpke_config.encode(&mut vec);
    STANDARD.encode(vec)
}

async fn set_up_schema_for<T: EntityTrait>(schema: &Schema, db: &Db, t: T) {
    let backend = db.get_database_backend();
    db.execute(backend.build(&schema.create_table_from_entity(t)))
        .await
        .unwrap();
}

async fn set_up_schema(db: &Db) {
    let schema = Schema::new(DbBackend::Sqlite);
    set_up_schema_for(&schema, db, Sessions).await;
    set_up_schema_for(&schema, db, Accounts).await;
    set_up_schema_for(&schema, db, Memberships).await;
    set_up_schema_for(&schema, db, Tasks).await;
    set_up_schema_for(&schema, db, queue::Entity).await;
    set_up_schema_for(&schema, db, Aggregators).await;
}

pub fn config(api_mocks: impl Handler) -> ApiConfig {
    ApiConfig {
        session_secret: "x".repeat(32),
        api_url: "https://api.example".parse().unwrap(),
        app_url: "https://app.example".parse().unwrap(),
        database_url: "sqlite::memory:".parse().unwrap(),
        auth_url: AUTH0_URL.parse().unwrap(),
        auth_client_id: "client id".into(),
        auth_client_secret: "client secret".into(),
        auth_audience: "aud".into(),
        prometheus_host: "localhost".into(),
        prometheus_port: 9464,
        postmark_token: "-".into(),
        email_address: "test@example.test".parse().unwrap(),
        postmark_url: POSTMARK_URL.parse().unwrap(),
        client: Client::new(trillium_testing::connector(api_mocks)),
        skip_app_compilation: false,
    }
}

pub async fn with_db<F, Fut>(f: F)
where
    F: FnOnce(Db) -> Fut,
    Fut: Future<Output = TestResult>,
{
    block_on(async move {
        let db = Db::connect("sqlite::memory").await;
        set_up_schema(&db).await;
        f(db).await.unwrap();
    })
}

pub async fn build_test_app() -> (DivviupApi, ClientLogs) {
    let api_mocks = ApiMocks::new();
    let client_logs = api_mocks.client_logs();
    let mut app = DivviupApi::new(config(api_mocks)).await;
    set_up_schema(app.db()).await;
    let mut info = "testing".into();
    app.init(&mut info).await;
    (app, client_logs)
}

pub fn set_up<F, Fut>(f: F)
where
    F: FnOnce(DivviupApi) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
    block_on(async move {
        let (app, _) = build_test_app().await;
        f(app).await.unwrap();
    });
}

pub fn with_client_logs<F, Fut>(f: F)
where
    F: FnOnce(DivviupApi, ClientLogs) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
    block_on(async move {
        let (app, client_logs) = build_test_app().await;
        f(app, client_logs).await.unwrap();
    });
}

pub const APP_CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

#[macro_export]
macro_rules! assert_not_found {
    ($conn:expr) => {
        assert_eq!($conn.status().unwrap_or(Status::NotFound), Status::NotFound);
        assert_eq!($conn.take_response_body_string().unwrap_or_default(), "");
    };
}

#[trillium::async_trait]
pub trait TestingJsonExt {
    async fn response_json<T: DeserializeOwned>(&mut self) -> T;
    fn with_request_json<T: Serialize>(self, t: T) -> Self;
}

#[trillium::async_trait]
impl TestingJsonExt for TestConn {
    async fn response_json<T: DeserializeOwned>(&mut self) -> T {
        assert_eq!(
            self.response_headers()
                .get_str(KnownHeaderName::ContentType)
                .unwrap(),
            APP_CONTENT_TYPE
        );

        let body = self
            .take_response_body()
            .expect("no body was set")
            .into_bytes()
            .await
            .expect("could not read body");

        serde_json::from_slice(&body).expect("could not deserialize body")
    }

    fn with_request_json<T: Serialize>(self, t: T) -> Self {
        self.with_request_body(serde_json::to_string(&t).unwrap())
    }
}

pub trait TestExt {
    fn with_api_headers(self) -> Self;
    fn with_api_host(self) -> Self;
    fn with_app_host(self) -> Self;
}

impl TestExt for TestConn {
    fn with_api_host(self) -> Self {
        self.with_request_header(KnownHeaderName::Host, "api.example")
            .secure()
    }

    fn with_app_host(self) -> Self {
        self.with_request_header(KnownHeaderName::Host, "app.example")
            .secure()
    }

    fn with_api_headers(self) -> Self {
        if self.method() == Method::Get {
            self
        } else {
            self.with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
        }
        .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
        .with_api_host()
    }
}

#[trillium::async_trait]
pub trait Reload: Sized {
    async fn reload(self, db: &impl ConnectionTrait) -> Result<Option<Self>, DbErr>;
}
