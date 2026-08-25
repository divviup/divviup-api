use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{MatchedPath, Request, State},
    http::Response,
    middleware::Next,
    response::IntoResponse,
    Error,
};
use http_body::{Frame, SizeHint};
use opentelemetry::{
    global,
    metrics::{Histogram, UpDownCounter},
    KeyValue,
};
use std::{
    pin::Pin,
    slice,
    task::{Context, Poll},
    time::Instant,
};

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
    request_body_size: Histogram<u64>,
    response_body_size: Histogram<u64>,
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
                    0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
                ])
                .build(),
            request_body_size: meter
                .u64_histogram("http.server.request.body.size")
                .with_description("Size of HTTP server request bodies.")
                .with_unit("By")
                .with_boundaries(BYTES_HISTOGRAM_BOUNDARIES.to_vec())
                .build(),
            response_body_size: meter
                .u64_histogram("http.server.response.body.size")
                .with_description("Size of HTTP server response bodies.")
                .with_unit("By")
                .with_boundaries(BYTES_HISTOGRAM_BOUNDARIES.to_vec())
                .build(),
            active_requests: meter
                .i64_up_down_counter("http.server.active_requests")
                .with_unit("{request}")
                .with_description("Number of active HTTP server requests")
                .build(),
        }
    }
}

/// These boundaries are intended to be used with measurements having the unit of "bytes".
pub const BYTES_HISTOGRAM_BOUNDARIES: &[f64] = &[
    1024.0, 2048.0, 4096.0, 8192.0, 16384.0, 32768.0, 65536.0, 131072.0, 262144.0, 524288.0,
    1048576.0, 2097152.0, 4194304.0, 8388608.0, 16777216.0, 33554432.0,
];

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
    let route_attr = route.map(|route| KeyValue::new("http.route", route));

    let active_attrs = [method_attr.clone(), scheme_attr.clone()];
    metrics.active_requests.add(1, &active_attrs);
    let _guard = ActiveGuard {
        counter: &metrics.active_requests,
        attrs: &active_attrs,
    };

    let (request_parts, request_body) = request.into_parts();
    let request = Request::from_parts(
        request_parts,
        Body::new(MeteredBody::new(
            request_body,
            metrics.request_body_size.clone(),
            route_attr.clone(),
        )),
    );

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();

    let (response_parts, response_body) = response.into_parts();
    let response = Response::from_parts(
        response_parts,
        Body::new(MeteredBody::new(
            response_body,
            metrics.response_body_size.clone(),
            route_attr.clone(),
        )),
    );

    let status = KeyValue::new(
        "http.response.status_code",
        i64::from(response.status().as_u16()),
    );
    let mut duration_attrs = vec![method_attr, scheme_attr, status];

    if let Some(route_attr) = route_attr {
        duration_attrs.push(route_attr);
    }
    metrics.request_duration.record(duration, &duration_attrs);

    response
}

/// Wrapper around [axum::body::Body] that keeps track of body sizes.
struct MeteredBody {
    /// The HTTP body we are wrapping.
    inner: Body,
    /// The number of bytes processed so far.
    total: usize,
    /// Whether we have recorded an observation of the body size to a metric.
    flushed: bool,
    /// The histogram metric to which we record an observation.
    histogram: Histogram<u64>,
    /// The `http.route` label and its value.
    route_attr: Option<KeyValue>,
}

impl MeteredBody {
    /// Wrap an HTTP body and record its size when finished.
    fn new(body: Body, histogram: Histogram<u64>, route_attr: Option<KeyValue>) -> Self {
        Self {
            inner: body,
            total: 0,
            flushed: false,
            histogram,
            route_attr,
        }
    }

    /// Record an observation of the body size to a histogram metric.
    ///
    /// This method keeps track of whether an observation has been recorded, so it is safe to call
    /// multiple times.
    fn record_metric(&mut self) {
        // Ensure we record an observation only once.
        if self.flushed {
            return;
        }
        self.flushed = true;

        let value = u64::try_from(self.total).unwrap_or(u64::MAX);
        let attrs = if let Some(attr) = &self.route_attr {
            slice::from_ref(attr)
        } else {
            &[]
        };
        self.histogram.record(value, attrs);
    }
}

impl HttpBody for MeteredBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let outcome = Pin::new(&mut self.inner).poll_frame(cx);
        match &outcome {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(bytes) = frame.data_ref() {
                    self.total += bytes.len();
                }
            }
            Poll::Ready(None) => {
                // At the end of the stream.
                self.record_metric();
            }
            _ => {}
        }
        outcome
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

impl Drop for MeteredBody {
    fn drop(&mut self) {
        self.record_metric();
    }
}
