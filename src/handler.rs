pub(crate) mod account_bearer_token;
#[cfg(assets)]
pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod misc;
pub(crate) mod oauth2;
pub(crate) mod opentelemetry;
pub(crate) mod origin_router;
pub(crate) mod session_store;

use crate::{routes, Config, Db};

use cors::cors_headers;
use error::ErrorHandler;
use logger::logger;
use session_store::SessionStore;
use std::{borrow::Cow, sync::Arc};
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
pub(crate) use misc::*;

pub use error::Error;
pub use origin_router::origin_router;

use self::opentelemetry::opentelemetry;

#[cfg(feature = "otlp-trace")]
use trillium_opentelemetry::global::instrument_handler;
#[cfg(not(feature = "otlp-trace"))]
fn instrument_handler(handler: impl Handler) -> impl Handler {
    handler
}

#[derive(Handler, Debug)]
pub struct DivviupApi {
    #[handler(except = init)]
    handler: Box<dyn Handler>,
    db: Db,
    config: Arc<Config>,
}

impl DivviupApi {
    pub async fn init(&mut self, info: &mut Info) {
        *info.server_description_mut() = format!("divviup-api {}", env!("CARGO_PKG_VERSION"));
        *info.listener_description_mut() = format!(
            "api url: {}\n             app url: {}\n",
            self.config.api_url, self.config.app_url,
        );
        self.handler.init(info).await
    }

    pub async fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let db = Db::connect(config.database_url.as_ref()).await;
        Self {
            handler: Box::new((
                conn_id(),
                routes::health_check(&db),
                Forwarding::trust_always(),
                opentelemetry(),
                caching_headers(),
                logger(),
                #[cfg(assets)]
                instrument_handler(assets::static_assets(&config)),
                instrument_handler(api(&db, &config)),
                ErrorHandler,
            )),
            db,
            config,
        }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn crypter(&self) -> &crate::Crypter {
        &self.config.crypter
    }
}

impl AsRef<Db> for DivviupApi {
    fn as_ref(&self) -> &Db {
        &self.db
    }
}

#[derive(Handler, Debug, Clone)]
pub struct NamedHandler<H>(#[handler(except = name)] H, Cow<'static, str>);
impl<H: Handler> NamedHandler<H> {
    fn name(&self) -> Cow<'static, str> {
        self.1.clone()
    }

    pub fn new(name: impl Into<Cow<'static, str>>, handler: H) -> Self {
        Self(handler, name.into())
    }
}

fn api(db: &Db, config: &Config) -> impl Handler {
    NamedHandler::new(
        "api",
        (
            instrument_handler(compression()),
            #[cfg(feature = "integration-testing")]
            state(crate::User::for_integration_testing()),
            instrument_handler(cookies()),
            instrument_handler(
                sessions(
                    SessionStore::new(db.clone()),
                    &config.session_secrets.current,
                )
                .with_cookie_name("divviup.sid")
                .with_older_secrets(&config.session_secrets.older),
            ),
            state(config.client.clone()),
            state(config.crypter.clone()),
            state(config.feature_flags()),
            instrument_handler(cors_headers(config)),
            cache_control([Private, MustRevalidate]),
            db.clone(),
            instrument_handler(routes(config)),
        ),
    )
}
