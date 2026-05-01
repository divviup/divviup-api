pub(crate) mod account_bearer_token;
#[cfg(assets)]
pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod extract;
pub(crate) mod logger;
pub(crate) mod misc;
pub(crate) mod oauth2;
pub(crate) mod opentelemetry;
pub(crate) mod origin_router;
pub(crate) mod session_store;

pub(crate) mod proxy;

use crate::{clients::Auth0Client, routes, routes::axum_routes, Config, Crypter, Db, FeatureFlags};

use axum::extract::{DefaultBodyLimit, FromRef};
use axum::http::{header, HeaderValue};
use cors::{axum_cors_layer, cors_headers};
use error::ErrorHandler;
use logger::logger;
use oauth2::OauthClient;
use proxy::AxumProxy;
use session_store::{axum_session_layer, SessionStore};
use std::{borrow::Cow, net::Ipv6Addr, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
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

/// Shared state for the Axum side of the application during migration.
#[derive(Clone, Debug)]
pub struct AxumAppState {
    pub(crate) db: Db,
    pub(crate) config: Arc<Config>,
    pub(crate) auth0_client: Auth0Client,
    pub(crate) oauth_client: OauthClient,
    pub(crate) crypter: Crypter,
    pub(crate) feature_flags: FeatureFlags,
}

impl FromRef<AxumAppState> for Db {
    fn from_ref(state: &AxumAppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AxumAppState> for Arc<Config> {
    fn from_ref(state: &AxumAppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AxumAppState> for Auth0Client {
    fn from_ref(state: &AxumAppState) -> Self {
        state.auth0_client.clone()
    }
}

impl FromRef<AxumAppState> for OauthClient {
    fn from_ref(state: &AxumAppState) -> Self {
        state.oauth_client.clone()
    }
}

impl FromRef<AxumAppState> for Crypter {
    fn from_ref(state: &AxumAppState) -> Self {
        state.crypter.clone()
    }
}

impl FromRef<AxumAppState> for FeatureFlags {
    fn from_ref(state: &AxumAppState) -> Self {
        state.feature_flags
    }
}

#[derive(Handler, Debug)]
pub struct DivviupApi {
    #[handler(except = init)]
    handler: Box<dyn Handler>,
    db: Db,
    config: Arc<Config>,
    axum_addr: SocketAddr,
}

impl DivviupApi {
    pub async fn init(&mut self, info: &mut Info) {
        *info.server_description_mut() = format!("divviup-api {}", env!("CARGO_PKG_VERSION"));
        *info.listener_description_mut() = format!(
            "api url: {}\n             app url: {}\n             axum: {}\n",
            self.config.api_url, self.config.app_url, self.axum_addr,
        );
        self.handler.init(info).await
    }

    pub async fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let db = Db::connect(config.database_url.as_ref()).await;

        // Spawn the Axum server on an ephemeral port. Routes will be migrated
        // here incrementally.
        let auth0_client = Auth0Client::new(&config);
        let axum_state = AxumAppState {
            db: db.clone(),
            config: config.clone(),
            auth0_client: auth0_client.clone(),
            oauth_client: OauthClient::new(&config.oauth_config()),
            crypter: config.crypter.clone(),
            feature_flags: config.feature_flags(),
        };
        // Middleware stack in logical order (outermost first), matching the
        // Trillium api() handler chain.
        let middleware = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(DefaultBodyLimit::max(1024 * 1024))
            .layer(CompressionLayer::new())
            .layer(SetResponseHeaderLayer::if_not_present(
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, must-revalidate"),
            ))
            .layer(axum_cors_layer(&config))
            .layer(axum_session_layer(db.clone(), &config.session_secrets));

        #[cfg(feature = "test-header-injection")]
        let middleware =
            middleware.layer(axum::middleware::from_fn(inject_integration_testing_user));

        let axum_router = axum::Router::new()
            // Temporary test endpoint to verify the proxy bridge works.
            // TODO: Remove once enough routes have migrated to make it redundant.
            .route(
                "/internal/test/axum_ready",
                axum::routing::get(|| async { "axum OK" }),
            )
            .route("/health", axum::routing::get(routes::health_check))
            .route("/login", axum::routing::get(oauth2::redirect))
            .route("/logout", axum::routing::get(oauth2::logout))
            .route("/callback", axum::routing::get(oauth2::callback))
            .nest("/api", axum_routes::api_router())
            .layer(middleware)
            .with_state(axum_state);
        let axum_listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0))
            .await
            .expect("failed to bind Axum listener on IPv6 loopback");
        let axum_addr = axum_listener
            .local_addr()
            .expect("failed to get Axum listener address");
        // TODO: Wire graceful shutdown into axum::serve(...).with_graceful_shutdown()
        // so that in-flight requests are drained when the Trillium server stops.
        tokio::spawn(async move {
            if let Err(e) = axum::serve(axum_listener, axum_router).await {
                log::error!("axum server error: {e}");
            }
        });

        let proxy = AxumProxy::new(axum_addr);

        Self {
            handler: Box::new((
                conn_id(),
                Forwarding::trust_always(),
                opentelemetry(),
                caching_headers(),
                logger(),
                #[cfg(assets)]
                instrument_handler(assets::static_assets(&config)),
                instrument_handler(api(&db, &config, auth0_client)),
                proxy,
                ErrorHandler,
            )),
            db,
            config,
            axum_addr,
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

    #[expect(dead_code)] // Scaffolded for later migration parts.
    pub(crate) fn axum_addr(&self) -> SocketAddr {
        self.axum_addr
    }
}

impl AsRef<Db> for DivviupApi {
    fn as_ref(&self) -> &Db {
        &self.db
    }
}

/// Trillium-side test shim that bridges user injection between the Trillium
/// and Axum worlds during the migration:
///
/// - If the `X-Integration-Testing-User` header is present (set by
///   `TestExt::with_user`), deserialize the user into connection state so
///   `actor_required` passes.
/// - If a `User` was injected via `.with_state()` (legacy test pattern),
///   serialize it into the header so the proxy forwards it to Axum.
#[cfg(feature = "test-header-injection")]
async fn inject_test_user_trillium(mut conn: trillium::Conn) -> trillium::Conn {
    if let Some(user) = conn
        .request_headers()
        .get_str("x-integration-testing-user")
        .and_then(|v| serde_json::from_str::<crate::User>(v).ok())
    {
        conn.insert_state(user);
    } else if let Some(json) = conn
        .state::<crate::User>()
        .and_then(|u| serde_json::to_string(u).ok())
    {
        conn.request_headers_mut()
            .insert("x-integration-testing-user", json);
    }
    conn
}

/// Axum-side test shim: if the request carries an `X-Integration-Testing-User`
/// header with a JSON-encoded [`crate::User`], place the user in request
/// extensions so [`crate::User`]'s extractor picks it up without a real
/// session.
///
/// Only compiled under `--features test-header-injection` (enabled by
/// `test-support`). Never compiled into deployed builds.
#[cfg(feature = "test-header-injection")]
async fn inject_integration_testing_user(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if let Some(user) = request
        .headers()
        .get("x-integration-testing-user")
        .and_then(|v| serde_json::from_slice::<crate::User>(v.as_bytes()).ok())
    {
        request.extensions_mut().insert(user);
    }
    next.run(request).await
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

fn api(db: &Db, config: &Config, auth0_client: Auth0Client) -> impl Handler {
    NamedHandler::new(
        "api",
        (
            instrument_handler(compression()),
            #[cfg(feature = "integration-testing")]
            state(crate::User::for_integration_testing()),
            #[cfg(feature = "test-header-injection")]
            inject_test_user_trillium,
            instrument_handler(cookies()),
            instrument_handler(
                sessions(
                    SessionStore::new(db.clone()),
                    &config.session_secrets.current,
                )
                .with_cookie_name(session_store::SESSION_COOKIE_NAME)
                .with_older_secrets(&config.session_secrets.older),
            ),
            state(config.client.clone()),
            state(config.crypter.clone()),
            state(config.feature_flags()),
            instrument_handler(cors_headers(config)),
            cache_control([Private, MustRevalidate]),
            db.clone(),
            instrument_handler(routes(auth0_client)),
        ),
    )
}
