pub(crate) mod account_bearer_token;
#[cfg(assets)]
pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod extract;
pub(crate) mod oauth2;
// TODO: remove origin_router in Part 9/10 (used by api_mocks)
pub(crate) mod origin_router;
pub(crate) mod session_store;

// TODO: remove proxy in Part 9 (only kept for DivviupApi test shim)
pub(crate) mod proxy;

use crate::{
    clients::{Auth0Client, HttpClient},
    routes::{axum_routes, health_check},
    Config, Crypter, Db, FeatureFlags,
};
use axum::{
    extract::{DefaultBodyLimit, FromRef},
    http::{header, HeaderValue},
    routing,
};
use cors::axum_cors_layer;
use error::ErrorHandler;
use oauth2::OauthClient;
// TODO: remove proxy + trillium imports in Part 9 (test-support rewrite)
use proxy::AxumProxy;
use session_store::axum_session_layer;
use std::{net::Ipv6Addr, sync::Arc};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer,
};
use trillium::{Handler, Info};
use trillium_macros::Handler;

pub use error::Error;
pub use origin_router::origin_router;

/// Shared state for the Axum application.
#[derive(Clone, Debug)]
pub struct AxumAppState {
    pub(crate) db: Db,
    pub(crate) config: Arc<Config>,
    pub(crate) auth0_client: Auth0Client,
    pub(crate) oauth_client: OauthClient,
    pub(crate) crypter: Crypter,
    pub(crate) feature_flags: FeatureFlags,
    pub(crate) client: HttpClient,
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

impl FromRef<AxumAppState> for HttpClient {
    fn from_ref(state: &AxumAppState) -> Self {
        state.client.clone()
    }
}

/// 1 MiB — this JSON API never needs bodies larger than this under normal operation.
const MAX_REQUEST_BODY_SIZE: usize = 1024 * 1024;

/// The result of [`build_app`]: an Axum router ready to serve, plus the
/// shared state that callers (e.g. the queue) need.
#[derive(Debug)]
pub struct BuiltApp {
    pub router: axum::Router,
    pub db: Db,
    pub config: Arc<Config>,
}

/// Build the Axum application router and connect to the database.
pub async fn build_app(config: Config) -> BuiltApp {
    let config = Arc::new(config);
    let db = Db::connect(config.database_url.as_ref()).await;

    let auth0_client = Auth0Client::new(&config);
    let axum_state = AxumAppState {
        db: db.clone(),
        config: config.clone(),
        auth0_client,
        oauth_client: OauthClient::new(&config.oauth_config()),
        crypter: config.crypter.clone(),
        feature_flags: config.feature_flags(),
        client: config.client.clone(),
    };

    // TODO(Part 9): add OpenTelemetry HTTP metrics middleware. The deleted
    // trillium-opentelemetry handler provided http.server.* histograms and
    // optional OTLP per-request spans; TraceLayer only emits tracing events.
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_SIZE))
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, must-revalidate"),
        ))
        .layer(axum_cors_layer(&config))
        .layer(axum_session_layer(db.clone(), &config.session_secrets));

    #[cfg(feature = "integration-testing")]
    let middleware = middleware.layer(axum::middleware::from_fn(inject_integration_testing_user));

    #[cfg(feature = "test-header-injection")]
    let middleware = middleware.layer(axum::middleware::from_fn(inject_test_header_user));

    #[cfg(assets)]
    let middleware = middleware.layer(axum::middleware::from_fn_with_state(
        assets::AssetConfig::new(&config.api_url, &config.app_url),
        assets::serve_assets,
    ));

    let router = axum::Router::new()
        .route("/health", routing::get(health_check))
        .route("/login", routing::get(oauth2::redirect))
        .route("/logout", routing::get(oauth2::logout))
        .route("/callback", routing::get(oauth2::callback))
        .nest("/api", axum_routes::api_router(&axum_state))
        .layer(middleware)
        .with_state(axum_state);

    BuiltApp { router, db, config }
}

// ---------------------------------------------------------------------------
// Test-only shim: DivviupApi
//
// test-support constructs a DivviupApi, calls .db()/.config()/.init(), and
// passes it to trillium_testing's .run_async(&app). This shim keeps that
// working by spawning the Axum router on a loopback port and proxying via
// AxumProxy.
//
// TODO: remove in Part 9 (test-support rewrite)
// ---------------------------------------------------------------------------

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
        self.handler.init(info).await
    }

    pub async fn new(config: Config) -> Self {
        let app = build_app(config).await;

        let axum_listener = TcpListener::bind((Ipv6Addr::LOCALHOST, 0))
            .await
            .expect("failed to bind Axum listener on IPv6 loopback");
        let axum_addr = axum_listener
            .local_addr()
            .expect("failed to get Axum listener address");
        tokio::spawn(async move {
            if let Err(e) = axum::serve(axum_listener, app.router).await {
                log::error!("axum server error: {e}");
            }
        });

        let proxy = AxumProxy::new(axum_addr);

        Self {
            handler: Box::new((
                #[cfg(feature = "test-header-injection")]
                inject_test_user_trillium,
                proxy,
                ErrorHandler,
            )),
            db: app.db,
            config: app.config,
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

// TODO: remove in Part 9 (test-support rewrite)
// NOTE: the CancellationToken created here is never cancelled, so workers
// spawned from this Queue would run forever. This is fine because callers
// only use perform_one_queue_job(), never spawn_workers().
impl From<&DivviupApi> for crate::Queue {
    fn from(app: &DivviupApi) -> Self {
        Self::new(app.db(), app.config(), CancellationToken::new())
    }
}

impl AsRef<Db> for DivviupApi {
    fn as_ref(&self) -> &Db {
        &self.db
    }
}

/// Trillium-side test shim: if a `User` was injected via `.with_state()`
/// (legacy test pattern), serialize it into the `X-Integration-Testing-User`
/// header so the proxy forwards it to Axum.
// TODO: remove in Part 9 (test-support rewrite — tests will set the header directly)
#[cfg(feature = "test-header-injection")]
async fn inject_test_user_trillium(mut conn: trillium::Conn) -> trillium::Conn {
    if let Some(json) = conn
        .state::<crate::User>()
        .and_then(|u| serde_json::to_string(u).ok())
    {
        conn.request_headers_mut()
            .insert("x-integration-testing-user", json);
    }
    conn
}

/// Axum middleware that unconditionally injects an admin
/// [`User`](crate::User) into every request. This is the Axum equivalent of
/// the Trillium `state(User::for_integration_testing())` that was in the old
/// `api()` handler chain.
///
/// Only compiled under `--features integration-testing` (enabled by
/// `compose.dev.override.yaml`). Never compiled into deployed builds.
#[cfg(feature = "integration-testing")]
async fn inject_integration_testing_user(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request
        .extensions_mut()
        .insert(crate::User::for_integration_testing());
    next.run(request).await
}

/// Axum middleware that reads an `X-Integration-Testing-User` header and
/// injects the decoded [`User`](crate::User) into request extensions.
/// Used by `test-support` to impersonate specific users in tests.
///
/// Only compiled under `--features test-header-injection` (enabled by
/// `test-support`). Never compiled into deployed builds.
#[cfg(feature = "test-header-injection")]
async fn inject_test_header_user(
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
