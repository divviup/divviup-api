#![allow(dead_code)] // because different tests use different parts of this
use divviup_api::{aggregator_api_mock::aggregator_api, ApiConfig, Db};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use trillium::Handler;
use trillium_testing::TestConn;

pub use divviup_api::{entity::*, DivviupApi, User};
pub use querystrong::QueryStrong;
pub use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DbBackend, EntityTrait, PaginatorTrait, Schema,
};
pub use serde_json::{json, Value};
pub use test_harness::test;
pub use trillium::KnownHeaderName;
pub use trillium_testing::prelude::*;
pub use url::Url;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn set_up_schema_for<T: EntityTrait>(schema: &Schema, db: &Db, t: T) {
    let backend = db.get_database_backend();
    db.execute(backend.build(&schema.create_table_from_entity(t)))
        .await
        .unwrap();
}

async fn set_up_schema(db: &Db) {
    let schema = Schema::new(DbBackend::Sqlite);
    set_up_schema_for(&schema, &db, Sessions).await;
    set_up_schema_for(&schema, &db, Accounts).await;
    set_up_schema_for(&schema, &db, Memberships).await;
    set_up_schema_for(&schema, &db, Tasks).await;
}

pub fn config(aggregator_url: Url) -> ApiConfig {
    ApiConfig {
        session_secret: std::iter::repeat('x').take(32).collect(),
        api_url: "https://api.example".parse().unwrap(),
        app_url: "https://app.example".parse().unwrap(),
        database_url: "sqlite::memory:".parse().unwrap(),
        auth_url: "https://auth.example".parse().unwrap(),
        auth_client_id: "client id".into(),
        auth_client_secret: "client secret".into(),
        auth_audience: "aud".into(),
        aggregator_url,
        aggregator_secret: "unused".into(),
        prometheus_host: "localhost".to_string(),
        prometheus_port: 9464,
    }
}

pub async fn build_test_app(aggregator_url: Url) -> DivviupApi {
    let mut app = DivviupApi::new(config(aggregator_url)).await;
    set_up_schema(app.db()).await;
    let mut info = "testing".into();
    app.init(&mut info).await;
    app
}

pub fn set_up<F, Fut>(f: F)
where
    F: FnOnce(DivviupApi) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
{
    block_on(async move {
        let app = build_test_app("https://aggregator_url.example".parse().unwrap()).await;
        f(app).await.unwrap();
    });
}

pub fn with_aggregator_api_mock<F, Fut>(f: F)
where
    F: FnOnce(DivviupApi) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
{
    trillium_testing::with_server(aggregator_api(), |aggregator_url| async move {
        let app = build_test_app(aggregator_url).await;
        f(app).await?;
        Ok(())
    });
}

pub mod fixtures {
    use divviup_api::{aggregator_api_mock, entity::task::HpkeConfig};
    use validator::Validate;

    use super::*;

    pub fn user() -> User {
        User {
            email: format!("test-{}@example.example", random_name()),
            email_verified: true,
            name: "test user".into(),
            nickname: "testy".into(),
            picture: None,
            sub: "".into(),
            updated_at: time::OffsetDateTime::now_utc(),
            admin: Some(false),
        }
    }

    pub fn random_name() -> String {
        std::iter::repeat_with(fastrand::alphabetic)
            .take(10)
            .collect()
    }

    pub async fn account(app: &DivviupApi) -> Account {
        Account::build(random_name())
            .unwrap()
            .insert(app.db())
            .await
            .unwrap()
    }

    pub async fn admin_account(app: &DivviupApi) -> Account {
        let mut active_model = Account::build(random_name()).unwrap();
        active_model.admin = ActiveValue::Set(true);
        active_model.insert(app.db()).await.unwrap()
    }

    pub async fn membership(app: &DivviupApi, account: &Account, user: &User) -> Membership {
        Membership::build(user.email.clone(), &account)
            .unwrap()
            .insert(app.db())
            .await
            .unwrap()
    }

    pub async fn admin(app: &DivviupApi) -> (User, Account, Membership) {
        let user = user();
        let account = admin_account(app).await;
        let membership = membership(app, &account, &user).await;

        (user, account, membership)
    }

    pub async fn member(app: &DivviupApi) -> (User, Account, Membership) {
        let user = user();
        let account = account(app).await;
        let membership = membership(app, &account, &user).await;

        (user, account, membership)
    }

    pub async fn task(app: &DivviupApi, account: &Account) -> Task {
        let new_task = NewTask {
            name: Some(random_name()),
            partner: Some(random_name()),
            vdaf: Some(task::Vdaf::Count),
            min_batch_size: Some(500),
            max_batch_size: Some(10000),
            is_leader: Some(true),
            expiration: None,
            time_precision_seconds: Some(60 * 60),
            hpke_config: Some(HpkeConfig {
                id: Some(1),
                kem_id: Some(1),
                kdf_id: Some(1),
                aead_id: Some(1),
                public_key: Some("stuff".into()),
            }),
        };
        new_task.validate().unwrap();
        let api_response = aggregator_api_mock::task_response(new_task.clone().into());
        task::build_task(new_task, api_response, &account)
            .insert(app.db())
            .await
            .unwrap()
    }
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

pub trait ApiHeadersExt {
    fn with_api_headers(self) -> Self;
}
impl ApiHeadersExt for TestConn {
    fn with_api_headers(self) -> Self {
        if self.method() == Method::Get {
            self.with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
        } else {
            self.with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
        }
    }
}
