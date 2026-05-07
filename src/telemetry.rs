use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
};
use git_version::git_version;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{MetricError, SdkMeterProvider},
    Resource,
};
use prometheus::{Encoder, Registry, TextEncoder};

/// Install the Prometheus metrics provider and return the [`Registry`] so it
/// can be shared with the metrics HTTP handler.
pub fn install_metrics() -> Result<Registry, MetricError> {
    let registry = Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let default_resource = Resource::default();

    let mut git_revision: &str = git_version!(fallback = "unknown");
    if git_revision == "unknown" {
        if let Some(value) = option_env!("GIT_REVISION") {
            git_revision = value;
        }
    }
    let version_info_resource = Resource::new([
        KeyValue::new("service.name", "divviup-api"),
        KeyValue::new(
            "service.version",
            format!("{}-{}", env!("CARGO_PKG_VERSION"), git_revision),
        ),
        KeyValue::new("process.runtime.name", "Rust"),
        KeyValue::new("process.runtime.version", env!("RUSTC_SEMVER")),
    ]);

    let resource = default_resource.merge(&version_info_resource);

    global::set_meter_provider(
        SdkMeterProvider::builder()
            .with_reader(exporter)
            .with_resource(resource.clone())
            .build(),
    );

    #[cfg(feature = "otlp-trace")]
    global::set_tracer_provider({
        use opentelemetry_otlp::SpanExporter;
        use opentelemetry_sdk::{runtime::Tokio, trace::TracerProvider};

        TracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(SpanExporter::builder().with_tonic().build().unwrap(), Tokio)
            .build()
    });

    Ok(registry)
}

/// Axum handler that serves Prometheus metrics in text format.
pub async fn metrics_handler(
    State(registry): State<Registry>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let encoder = TextEncoder::new();
    let content_type = encoder.format_type().to_owned();
    let metrics = registry.gather();
    let mut buf = String::new();
    encoder
        .encode_utf8(&metrics, &mut buf)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(([(header::CONTENT_TYPE, content_type)], buf))
}
