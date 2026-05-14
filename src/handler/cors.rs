use crate::Config;
use axum::http::{header, HeaderValue, Method as HttpMethod};
use time::Duration;
use tower_http::cors::CorsLayer;

/// Build a [`tower_http::cors::CorsLayer`] for the Axum router.
pub fn axum_cors_layer(config: &Config) -> CorsLayer {
    let origin = config
        .app_url
        .as_str()
        .strip_suffix('/')
        .unwrap_or(config.app_url.as_str());

    CorsLayer::new()
        // unwrap safety: config.app_url is a valid URL, so stripping the
        // trailing slash still leaves a valid HTTP header value.
        .allow_origin(
            origin
                .parse::<HeaderValue>()
                .expect("config.app_url must be a valid header value"),
        )
        .allow_methods([
            HttpMethod::POST,
            HttpMethod::DELETE,
            HttpMethod::OPTIONS,
            HttpMethod::GET,
            HttpMethod::PATCH,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::IF_NONE_MATCH,
            header::IF_MODIFIED_SINCE,
            header::ETAG,
        ])
        .allow_credentials(true)
        .max_age(Duration::DAY.unsigned_abs())
}
