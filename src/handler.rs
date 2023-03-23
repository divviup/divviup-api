pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod misc;
pub(crate) mod oauth2;
pub(crate) mod session_store;

use crate::{routes, AggregatorClient, ApiConfig, Db};
use cors::cors_headers;
use error::ErrorHandler;
use logger::logger;
use session_store::SessionStore;
use std::sync::Arc;
use trillium::{init, state, Handler};
use trillium_caching_headers::{
    cache_control, caching_headers,
    CacheControlDirective::{MustRevalidate, Private},
};
use trillium_compression::compression;
use trillium_conn_id::conn_id;
use trillium_cookies::cookies;
use trillium_sessions::sessions;

pub(crate) use custom_mime_types::ReplaceMimeTypes;
pub(crate) use error::Error;
pub(crate) use misc::*;

pub fn divviup_api(config: ApiConfig) -> impl Handler {
    init(move |_| {
        let config = config.clone();
        async move {
            let config = Arc::new(config);
            let db = Db::connect(&config).await;
            let aggregator_client = AggregatorClient::new(&config);

            (
                db.clone(),
                compression(),
                conn_id(),
                state(config.clone()),
                state(aggregator_client),
                cors_headers,
                logger(),
                caching_headers(),
                cache_control([Private, MustRevalidate]),
                cookies(),
                sessions(SessionStore::new(db), config.session_secret.clone())
                    .with_cookie_name("divviup.sid"),
                routes(&config),
                ErrorHandler,
            )
        }
    })
}
