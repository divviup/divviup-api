use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
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
    serve_dir: ServeDir,
    serve_index: ServeFile,
}

impl AssetConfig {
    pub fn new(api_url: &Url, app_url: &Url) -> Self {
        // TODO(#2263): move ASSET_DIR from compile-time to runtime env var
        let asset_dir = PathBuf::from(env!("ASSET_DIR"));
        let serve_index = ServeFile::new(asset_dir.join("index.html"));
        let serve_dir = ServeDir::new(asset_dir);
        let host = app_url.host_str().expect("app_url must have a host");
        let app_host = match app_url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_owned(),
        };
        Self {
            api_url: api_url.clone(),
            app_host,
            serve_dir,
            serve_index,
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
}

/// Axum middleware that serves the React SPA for requests whose `Host`
/// (or `X-Forwarded-Host`) matches the configured app origin. Requests
/// to other hosts pass through to the API routes.
pub async fn serve_assets(
    State(config): State<AssetConfig>,
    request: Request,
    next: Next,
) -> Response {
    let host_matches =
        request_host(request.headers()).is_some_and(|h| h.eq_ignore_ascii_case(&config.app_host));

    if !host_matches {
        return next.run(request).await;
    }

    if request.uri().path() == "/api_url" {
        let headers = [
            (header::CONTENT_TYPE, "text/plain"),
            (header::CACHE_CONTROL, "no-cache"),
        ];
        return if request.method() == Method::HEAD {
            (headers, "").into_response()
        } else {
            (headers, config.api_url.to_string()).into_response()
        };
    }

    let is_asset_path = request.uri().path().starts_with("/assets");

    let mut response = config.serve_dir.clone().call(request).await.into_response();

    if response.status() == StatusCode::NOT_FOUND && !is_asset_path {
        let fallback_request = Request::new(Body::empty());
        response = config
            .serve_index
            .clone()
            .call(fallback_request)
            .await
            .into_response();
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
