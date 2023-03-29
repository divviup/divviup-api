use divviup_api::{aggregator_api_mock::aggregator_api, entity::*, ApiConfig, Db, DivviupApi};
use sea_orm::{ConnectionTrait, DbBackend, Schema};
use std::future::Future;
use trillium::Handler;
use url::Url;

async fn set_up_schema(db: &Db) {
    let schema = Schema::new(DbBackend::Sqlite);
    db.execute(
        db.get_database_backend()
            .build(&schema.create_table_from_entity(Sessions)),
    )
    .await
    .unwrap();

    db.execute(
        db.get_database_backend()
            .build(&schema.create_table_from_entity(Accounts)),
    )
    .await
    .unwrap();

    db.execute(
        db.get_database_backend()
            .build(&schema.create_table_from_entity(Memberships)),
    )
    .await
    .unwrap();

    db.execute(
        db.get_database_backend()
            .build(&schema.create_table_from_entity(Tasks)),
    )
    .await
    .unwrap();
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

pub fn with_server<F, Fut>(f: F)
where
    F: FnOnce(DivviupApi) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    trillium_testing::with_server(aggregator_api(), |aggregator_url| async move {
        f(build_test_app(aggregator_url).await).await;
        Ok(())
    });
}
