use std::sync::Arc;

use opentelemetry::{
    metrics::MetricsError,
    sdk::{
        export::metrics::aggregation::stateless_temporality_selector,
        metrics::{controllers, processors, selectors::simple::histogram},
    },
};
use opentelemetry_prometheus::{Encoder, TextEncoder};
use tokio::spawn;
use trillium::{KnownHeaderName, Status};
use trillium_router::Router;

/// Install a Prometheus metrics provider and exporter. The OpenTelemetry global API can be used to
/// create and update instruments, and they will be sent through this exporter.
pub fn install_metrics_exporter(host: &str, port: u16) -> Result<(), MetricsError> {
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

    let router = Router::new().get("metrics", move |conn: trillium::Conn| {
        let exporter = Arc::clone(&exporter);
        async move {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            match encoder.encode(&exporter.registry().gather(), &mut buffer) {
                Ok(()) => conn
                    .with_header(
                        KnownHeaderName::ContentType,
                        encoder.format_type().to_owned(),
                    )
                    .ok(buffer),
                Err(_) => conn.with_status(Status::InternalServerError),
            }
        }
    });

    spawn(
        trillium_tokio::config()
            .with_host(host)
            .with_port(port)
            .without_signals()
            .run_async(router),
    );

    Ok(())
}
