pub(crate) mod account_bearer_token;
#[cfg(assets)]
pub(crate) mod assets;
pub(crate) mod cors;
pub(crate) mod custom_mime_types;
pub(crate) mod error;
pub(crate) mod extract;
pub(crate) mod http_metrics;
pub(crate) mod oauth2;
pub(crate) mod session_store;

use crate::{
    clients::{Auth0Client, HttpClient},
    routes::{axum_routes, health_check},
    Config, Crypter, Db, FeatureFlags,
};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, FromRef},
    http::{header, HeaderValue, Request},
    routing,
};
use cors::axum_cors_layer;
use http_metrics::HttpMetrics;
use oauth2::OauthClient;
use session_store::axum_session_layer;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer,
};

pub use error::Error;

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

    let middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn_with_state(
            HttpMetrics::new(),
            http_metrics::http_metrics_middleware,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let client_ip = request
                    .headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.rsplit(',').next())
                    .map(str::trim)
                    .unwrap_or("");
                tracing::debug_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    client.address = client_ip,
                )
            }),
        )
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

/// Axum middleware that injects an admin [`User`](crate::User) into every
/// request that doesn't already have one in extensions.
///
/// Only compiled under `--features integration-testing` (enabled by
/// `compose.dev.override.yaml`). Never compiled into deployed builds.
#[cfg(feature = "integration-testing")]
async fn inject_integration_testing_user(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if request.extensions().get::<crate::User>().is_none() {
        request
            .extensions_mut()
            .insert(crate::User::for_integration_testing());
    }
    next.run(request).await
}
