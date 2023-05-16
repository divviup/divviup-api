use git_version::git_version;
use opentelemetry::{
    global,
    metrics::MetricsError,
    sdk::{
        export::metrics::aggregation::stateless_temporality_selector,
        metrics::{controllers, processors, selectors::simple::histogram},
    },
    Context, KeyValue,
};
use std::sync::Arc;

/// Install a Prometheus metrics provider and exporter. The
/// OpenTelemetry global API can be used to create and update
/// instruments, and they will be sent through this exporter.
pub fn metrics_exporter() -> Result<impl trillium::Handler, MetricsError> {
    let exporter = Arc::new(
        opentelemetry_prometheus::exporter(
            controllers::basic(processors::factory(
                histogram([
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ]),
                stateless_temporality_selector(),
            ))
            .build(),
        )
        .try_init()?,
    );

    // Record the binary's version information in a build info metric.
    let meter = global::meter("divviup-api");
    let gauge = meter
        .u64_observable_gauge("divviup_api_build_info")
        .with_description("Build-time version information")
        .init();
    let mut git_revision: &str = git_version!(fallback = "unknown");
    if git_revision == "unknown" {
        if let Some(value) = option_env!("GIT_REVISION") {
            git_revision = value;
        }
    }
    gauge.observe(
        &Context::current(),
        1,
        &[
            KeyValue::new("version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("revision", git_revision),
            KeyValue::new("rust_version", env!("RUSTC_SEMVER")),
        ],
    );

    Ok(trillium_prometheus::text_format_handler(
        exporter.registry().clone(),
    ))
}
