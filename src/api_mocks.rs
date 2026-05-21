use axum::{
    extract::{Request, State},
    response::IntoResponse,
    Router,
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;
use url::Url;

pub mod aggregator_api;
pub mod auth0;
pub mod postmark;

fn random_chars(n: usize) -> String {
    std::iter::repeat_with(fastrand::alphabetic)
        .take(n)
        .collect()
}

#[derive(Clone)]
struct HostMap {
    routers: HashMap<String, Router>,
    fallback: Router,
}

#[derive(Debug)]
pub struct ApiMocks {
    router: Router,
}

impl ApiMocks {
    pub fn new(postmark_url: &str, auth0_url: &str) -> Self {
        let mut routers = HashMap::new();
        routers.insert(extract_host(postmark_url), postmark::mock());
        routers.insert(extract_host(auth0_url), auth0::mock(auth0_url));

        let host_map = Arc::new(HostMap {
            routers,
            fallback: aggregator_api::mock(),
        });

        let router = Router::new()
            .fallback(dispatch_by_host)
            .with_state(host_map);

        Self { router }
    }

    pub fn into_router(self) -> Router {
        self.router
    }
}

// NB: Url::host_str() strips IPv6 brackets, but the Host header keeps them.
// Current callers only pass hostname URLs, so this doesn't bite today.
fn extract_host(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str().map(|h| match u.port() {
                Some(p) => format!("{h}:{p}"),
                None => h.to_string(),
            })
        })
        .unwrap_or_default()
        .to_lowercase()
}

async fn dispatch_by_host(State(host_map): State<Arc<HostMap>>, req: Request) -> impl IntoResponse {
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let router = host_map
        .routers
        .get(&host)
        .unwrap_or(&host_map.fallback)
        .clone();

    router.oneshot(req).await.into_response()
}
