pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod misc;
pub(crate) mod oauth2;
pub(crate) mod origin_router;
pub(crate) mod session_store;

use crate::{clients::AggregatorClient, routes, ApiConfig, Db};

use assets::static_assets;
use cors::cors_headers;
use error::ErrorHandler;
use logger::logger;
use session_store::SessionStore;
use std::sync::Arc;
use trillium::{state, Handler};
use trillium_caching_headers::{
    cache_control, caching_headers,
    CacheControlDirective::{MustRevalidate, Private},
};
use trillium_compression::compression;
use trillium_conn_id::conn_id;
use trillium_cookies::cookies;
use trillium_forwarding::Forwarding;
use trillium_macros::Handler;
use trillium_sessions::sessions;

pub(crate) use custom_mime_types::ReplaceMimeTypes;
pub(crate) use error::Error;
pub(crate) use misc::*;

pub use origin_router::origin_router;

#[derive(Handler, Debug)]
pub struct DivviupApi {
    #[handler]
    handler: Box<dyn Handler>,
    db: Db,
    config: Arc<ApiConfig>,
}

impl DivviupApi {
    pub async fn new(config: ApiConfig) -> Self {
        let config = Arc::new(config);
        let db = Db::connect(config.database_url.as_ref()).await;
        Self {
            handler: Box::new((
                Forwarding::trust_always(),
                caching_headers(),
                conn_id(),
                logger(),
                origin_router()
                    .with_handler(config.app_url.as_ref(), static_assets(&config))
                    .with_handler(config.api_url.as_ref(), api(&db, &config)),
                ErrorHandler,
            )),
            db,
            config,
        }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn config(&self) -> &ApiConfig {
        &self.config
    }
}

fn api(db: &Db, config: &ApiConfig) -> impl Handler {
    let aggregator_client = AggregatorClient::new(config);
    (
        compression(),
        #[cfg(feature = "kind-integration")]
        state(crate::User::for_kind()),
        cookies(),
        sessions(SessionStore::new(db.clone()), config.session_secret.clone())
            .with_cookie_name("divviup.sid"),
        state(aggregator_client),
        cors_headers(config),
        cache_control([Private, MustRevalidate]),
        db.clone(),
        routes(config),
    )
}
