#![allow(dead_code)] // because different tests use different parts of this
use divviup_api::{aggregator_api_mock::aggregator_api, ApiConfig, Db, DivviupApi};
pub use divviup_api::{entity::*, User};
pub use querystrong::QueryStrong;
pub use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, DbBackend, EntityTrait, Schema};
use serde::de::DeserializeOwned;
use std::future::Future;
use trillium::Handler;
pub use trillium::KnownHeaderName;
pub use trillium_testing::prelude::*;
pub use url::Url;

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
    trillium_testing::with_server(aggregator_api(), |aggregator_url| async move {
        let app = build_test_app(aggregator_url).await;
        f(app).await?;
        Ok(())
    });
}

pub fn test_user() -> User {
    User {
        email: "test@example.example".into(),
        email_verified: true,
        name: "test user".into(),
        nickname: "testy".into(),
        picture: None,
        sub: "".into(),
        updated_at: time::OffsetDateTime::now_utc(),
        admin: Some(false),
    }
}

pub fn build_admin_account(name: &str) -> account::ActiveModel {
    let mut account = Account::build(name.into()).expect("could not validate account");
    account.admin = ActiveValue::Set(true);
    account
}

pub const APP_CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

pub async fn json_response<T: DeserializeOwned>(conn: &mut Conn) -> T {
    assert_eq!(
        conn.response_headers()
            .get_str(KnownHeaderName::ContentType)
            .unwrap(),
        APP_CONTENT_TYPE
    );

    let body = conn
        .take_response_body()
        .expect("no body was set")
        .into_bytes()
        .await
        .expect("could not read body");

    serde_json::from_slice(&body).expect("could not deserialize body")
}
