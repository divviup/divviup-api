use git_version::git_version;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{MetricError, SdkMeterProvider},
    Resource,
};
use prometheus::Registry;

/// Install a Prometheus metrics provider and exporter. The
/// OpenTelemetry global API can be used to create and update
/// instruments, and they will be sent through this exporter.
pub fn metrics_exporter() -> Result<impl trillium::Handler, MetricError> {
    let registry = Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    // Note that the implementation of `Default` pulls in attributes set via environment variables.
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
        use opentelemetry_sdk::{runtime::Tokio, trace::SdkTracerProvider};

        SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(SpanExporter::builder().with_tonic().build().unwrap(), Tokio)
            .build()
    });

    Ok(trillium_prometheus::text_format_handler(registry))
}
