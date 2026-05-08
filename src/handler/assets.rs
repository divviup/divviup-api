use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tower::Service;
use tower_http::services::{ServeDir, ServeFile};
use url::Url;

#[derive(Clone, Debug)]
pub struct AssetConfig {
    pub api_url: Url,
    pub app_host: String,
    asset_dir: PathBuf,
    index_file: PathBuf,
}

impl AssetConfig {
    pub fn new(api_url: &Url, app_url: &Url) -> Self {
        let asset_dir = PathBuf::from(env!("ASSET_DIR"));
        let index_file = asset_dir.join("index.html");
        Self {
            api_url: api_url.clone(),
            app_host: app_url
                .host_str()
                .expect("app_url must have a host")
                .to_owned(),
            asset_dir,
            index_file,
        }
    }
}

/// Assumes we are behind a reverse proxy that strips client-supplied
/// `X-Forwarded-Host` before injecting its own.
fn request_host(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h))
}

/// Axum middleware that serves the React SPA for requests whose `Host`
/// (or `X-Forwarded-Host`) matches the configured app origin. Requests
/// to other hosts pass through to the API routes.
pub async fn serve_assets(
    State(config): State<AssetConfig>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let host_matches =
        request_host(request.headers()).is_some_and(|h| h.eq_ignore_ascii_case(&config.app_host));

    if !host_matches {
        return next.run(request).await;
    }

    if request.uri().path() == "/api_url" {
        return (
            [
                (header::CONTENT_TYPE, "text/plain"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            config.api_url.to_string(),
        )
            .into_response();
    }

    let is_asset_path = request.uri().path().starts_with("/assets");

    let mut service = ServeDir::new(&config.asset_dir);
    let mut response = service.call(request).await.into_response();

    if response.status() == StatusCode::NOT_FOUND && !is_asset_path {
        let fallback_request = axum::extract::Request::default();
        let mut fallback = ServeFile::new(&config.index_file);
        response = fallback.call(fallback_request).await.into_response();
    }

    let cache_value = if response.status().is_success() && is_asset_path {
        HeaderValue::from_static("max-age=31536000")
    } else {
        HeaderValue::from_static("no-cache")
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, cache_value);
    response
}
