use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    convert::Infallible,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use tower_http::services::{ServeDir, ServeFile};
use url::Url;

#[derive(Clone, Debug)]
pub struct AssetConfig {
    pub api_url: Url,
    pub app_host: String,
    serve_dir: ServeDir<IndexFallback>,
}

impl AssetConfig {
    pub fn new(api_url: &Url, app_url: &Url) -> Self {
        // TODO(#2263): move ASSET_DIR from compile-time to runtime env var
        let asset_dir = PathBuf::from(env!("ASSET_DIR"));
        let fallback = IndexFallback::new(asset_dir.join("index.html"));
        let serve_dir = ServeDir::new(asset_dir).fallback(fallback);
        let host = app_url.host_str().expect("app_url must have a host");
        let app_host = match app_url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_owned(),
        };
        Self {
            api_url: api_url.clone(),
            app_host,
            serve_dir,
        }
    }
}

/// Fallback service for `ServeDir` that serves `index.html` for
/// non-asset paths (SPA routing) and returns 404 for `/assets/*` misses.
#[derive(Clone, Debug)]
struct IndexFallback {
    serve_index: ServeFile,
}

impl IndexFallback {
    fn new(index_path: impl AsRef<Path>) -> Self {
        Self {
            serve_index: ServeFile::new(index_path),
        }
    }
}

impl Service<Request<Body>> for IndexFallback {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<Request<Body>>::poll_ready(&mut self.serve_index, cx)
            .map_err(|e: Infallible| match e {})
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if req.uri().path().starts_with("/assets") {
            Box::pin(async { Ok(StatusCode::NOT_FOUND.into_response()) })
        } else {
            let future = self.serve_index.call(req);
            Box::pin(async { Ok(future.await.into_response()) })
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
