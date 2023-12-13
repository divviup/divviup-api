use git_version::git_version;
use opentelemetry::{global, metrics::MetricsError, KeyValue};
use opentelemetry_sdk::{metrics::MeterProvider, Resource};
use prometheus::Registry;

/// Install a Prometheus metrics provider and exporter. The
/// OpenTelemetry global API can be used to create and update
/// instruments, and they will be sent through this exporter.
pub fn metrics_exporter() -> Result<impl trillium::Handler, MetricsError> {
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
        MeterProvider::builder()
            .with_reader(exporter)
            .with_resource(resource.clone())
            .build(),
    );

    #[cfg(feature = "otlp-trace")]
    global::set_tracer_provider(
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(resource))
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap()
            .provider()
            .unwrap(),
    );

    Ok(trillium_prometheus::text_format_handler(registry))
}
