use crate::Config;
use axum::http::{header, HeaderValue, Method as HttpMethod};
use time::Duration;
use tower_http::cors::CorsLayer;
use trillium::{
    Conn, Handler,
    KnownHeaderName::{
        AccessControlAllowCredentials, AccessControlAllowHeaders, AccessControlAllowMethods,
        AccessControlAllowOrigin, AccessControlMaxAge, Origin,
    },
    Method, Status,
};

#[derive(Debug)]
struct CorsHeaders {
    origin: String,
}

#[trillium::async_trait]
impl Handler for CorsHeaders {
    async fn run(&self, mut conn: Conn) -> Conn {
        let conn_origin = conn.request_headers().get_str(Origin);

        if conn_origin == Some(&self.origin) {
            conn.response_headers_mut().extend([
                (
                    AccessControlAllowMethods,
                    "POST, DELETE, OPTIONS, GET, PATCH",
                ),
                (AccessControlAllowCredentials, "true"),
                (
                    AccessControlAllowHeaders,
                    "Content-Type, If-None-Match, If-Modified-Since, Etag",
                ),
                (AccessControlMaxAge, "86400"),
            ]);
            conn.response_headers_mut()
                .insert(AccessControlAllowOrigin, self.origin.clone());

            if conn.method() == Method::Options {
                return conn.with_status(Status::NoContent).halt();
            }
        }
        conn
    }
}

impl CorsHeaders {
    pub fn new(config: &Config) -> Self {
        let mut origin = config.app_url.to_string();
        origin.pop();
        Self { origin }
    }
}

pub fn cors_headers(config: &Config) -> impl Handler {
    CorsHeaders::new(config)
}

/// Build a [`tower_http::cors::CorsLayer`] matching the Trillium [`CorsHeaders`]
/// behavior above.
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
