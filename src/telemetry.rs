use git_version::git_version;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};

#[cfg(feature = "otlp-trace")]
use opentelemetry_sdk::trace::SdkTracerProvider;

#[derive(Debug)]
pub struct TelemetryProviders {
    meter_provider: SdkMeterProvider,
    #[cfg(feature = "otlp-trace")]
    tracer_provider: SdkTracerProvider,
}

impl TelemetryProviders {
    pub fn shutdown(self) {
        #[cfg(feature = "otlp-trace")]
        if let Err(e) = self.tracer_provider.shutdown() {
            tracing::error!("tracer provider shutdown error: {e}");
        }
        if let Err(e) = self.meter_provider.shutdown() {
            tracing::error!("meter provider shutdown error: {e}");
        }
    }
}

/// Install OTLP telemetry providers. Metrics (and optionally traces) are
/// pushed to the collector endpoint configured via
/// `OTEL_EXPORTER_OTLP_ENDPOINT` (default `http://localhost:4318`).
///
/// Returns [`TelemetryProviders`] so callers can shut down gracefully.
pub fn install_telemetry() -> Result<TelemetryProviders, Box<dyn std::error::Error>> {
    let exporter = MetricExporter::builder().with_http().build()?;

    let mut git_revision: &str = git_version!(fallback = "unknown");
    if git_revision == "unknown" {
        if let Some(value) = option_env!("GIT_REVISION") {
            git_revision = value;
        }
    }

    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", "divviup-api"),
            KeyValue::new(
                "service.version",
                format!("{}-{}", env!("CARGO_PKG_VERSION"), git_revision),
            ),
            KeyValue::new("process.runtime.name", "Rust"),
            KeyValue::new("process.runtime.version", env!("RUSTC_SEMVER")),
        ])
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource.clone())
        .build();

    global::set_meter_provider(meter_provider.clone());

    #[cfg(feature = "otlp-trace")]
    let tracer_provider = {
        use opentelemetry_otlp::SpanExporter;

        let provider = SdkTracerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(
                SpanExporter::builder().with_http().build()?,
            )
            .build();
        global::set_tracer_provider(provider.clone());
        provider
    };

    Ok(TelemetryProviders {
        meter_provider,
        #[cfg(feature = "otlp-trace")]
        tracer_provider,
    })
}
