pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod misc;
pub(crate) mod oauth2;
pub(crate) mod origin_router;
pub(crate) mod session_store;

use crate::{routes, ApiConfig, Db};

use assets::static_assets;
use cors::cors_headers;
use error::ErrorHandler;
use logger::logger;
use session_store::SessionStore;
use std::sync::Arc;
use trillium::{state, Handler, Info};
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
    #[handler(except = init)]
    handler: Box<dyn Handler>,
    db: Db,
    config: Arc<ApiConfig>,
}

impl DivviupApi {
    async fn init(&mut self, info: &mut Info) {
        *info.server_description_mut() = format!("divviup-api {}", env!("CARGO_PKG_VERSION"));
        *info.listener_description_mut() = format!(
            "api url: {}\n             app url: {}\n",
            self.config.api_url, self.config.app_url,
        );
        self.handler.init(info).await
    }

    pub async fn new(config: ApiConfig) -> Self {
        let config = Arc::new(config);
        let db = Db::connect(config.database_url.as_ref()).await;
        Self {
            handler: Box::new((
                conn_id(),
                routes::health_check(&db),
                Forwarding::trust_always(),
                caching_headers(),
                logger(),
                static_assets(&config),
                api(&db, &config),
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

impl AsRef<Db> for DivviupApi {
    fn as_ref(&self) -> &Db {
        &self.db
    }
}

fn api(db: &Db, config: &ApiConfig) -> impl Handler {
    (
        compression(),
        #[cfg(feature = "integration-testing")]
        state(crate::User::for_integration_testing()),
        cookies(),
        sessions(SessionStore::new(db.clone()), config.session_secret.clone())
            .with_cookie_name("divviup.sid"),
        state(config.client.clone()),
        cors_headers(config),
        cache_control([Private, MustRevalidate]),
        db.clone(),
        routes(config),
    )
}
