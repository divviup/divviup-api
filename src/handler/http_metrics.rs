use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::IntoResponse,
};
use opentelemetry::{
    global,
    metrics::{Histogram, UpDownCounter},
    KeyValue,
};
use std::time::Instant;

fn normalize_method(method: &str) -> &str {
    match method {
        "CONNECT" | "DELETE" | "GET" | "HEAD" | "OPTIONS" | "PATCH" | "POST" | "PUT" | "TRACE" => {
            method
        }
        _ => "_OTHER",
    }
}

fn request_scheme(request: &Request) -> &str {
    request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .or_else(|| request.uri().scheme_str())
        .unwrap_or("http")
}

struct ActiveGuard<'a> {
    counter: &'a UpDownCounter<i64>,
    attrs: &'a [KeyValue],
}

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.counter.add(-1, self.attrs);
    }
}

#[derive(Clone)]
pub struct HttpMetrics {
    request_duration: Histogram<f64>,
    active_requests: UpDownCounter<i64>,
}

impl Default for HttpMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMetrics {
    pub fn new() -> Self {
        let meter = global::meter("divviup-api");
        Self {
            request_duration: meter
                .f64_histogram("http.server.request.duration")
                .with_unit("s")
                .with_description("Duration of HTTP server requests")
                .with_boundaries(vec![
                    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0,
                    7.5, 10.0,
                ])
                .build(),
            active_requests: meter
                .i64_up_down_counter("http.server.active_requests")
                .with_unit("{request}")
                .with_description("Number of active HTTP server requests")
                .build(),
        }
    }
}

pub async fn http_metrics_middleware(
    State(metrics): State<HttpMetrics>,
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let method = normalize_method(request.method().as_str());
    let scheme = request_scheme(&request);
    let route = matched_path.map(|p| p.as_str().to_owned());

    let method_attr = KeyValue::new("http.request.method", method.to_owned());
    let scheme_attr = KeyValue::new("url.scheme", scheme.to_owned());

    let active_attrs = [method_attr.clone(), scheme_attr.clone()];
    metrics.active_requests.add(1, &active_attrs);
    let _guard = ActiveGuard {
        counter: &metrics.active_requests,
        attrs: &active_attrs,
    };

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();

    let status_code = response.status();
    let status = KeyValue::new(
        "http.response.status_code",
        i64::from(status_code.as_u16()),
    );
    let mut duration_attrs = vec![method_attr, scheme_attr, status];
    if status_code.is_server_error() || status_code.is_client_error() {
        duration_attrs.push(KeyValue::new(
            "error.type",
            status_code.as_u16().to_string(),
        ));
    }
    if let Some(route) = route {
        duration_attrs.push(KeyValue::new("http.route", route));
    }
    metrics.request_duration.record(duration, &duration_attrs);

    response
}
