use git_version::git_version;
use opentelemetry::{
    global,
    metrics::MetricsError,
    sdk::{metrics::MeterProvider, Resource},
    KeyValue,
};
use prometheus::Registry;

/// Install a Prometheus metrics provider and exporter. The
/// OpenTelemetry global API can be used to create and update
/// instruments, and they will be sent through this exporter.
pub fn metrics_exporter() -> Result<impl trillium::Handler, MetricsError> {
    let registry = Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()?;

    // Note that the implementation of `Default` pulls in attributes set via environment variables.
    let default_resource = Resource::default();

    let mut git_revision: &str = git_version!(fallback = "unknown");
    if git_revision == "unknown" {
        if let Some(value) = option_env!("GIT_REVISION") {
            git_revision = value;
        }
    }
    let version_info_resource = Resource::new([
        KeyValue::new(
            "service.version",
            format!("{}-{}", env!("CARGO_PKG_VERSION"), git_revision),
        ),
        KeyValue::new("process.runtime.name", "Rust"),
        KeyValue::new("process.runtime.version", env!("RUSTC_SEMVER")),
    ]);

    let resource = version_info_resource.merge(&default_resource);

    let meter_provider = MeterProvider::builder()
        .with_reader(exporter)
        .with_resource(resource)
        .build();
    global::set_meter_provider(meter_provider);

    Ok(trillium_prometheus::text_format_handler(registry))
}
