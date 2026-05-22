use divviup_api::{
    handler::build_app, telemetry, trace, trace::install_trace_subscriber, Config, Queue,
};
use rustls::crypto::aws_lc_rs;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
struct MonitoringState {
    trace_reload_handle: Arc<trace::TraceReloadHandle>,
}

impl axum::extract::FromRef<MonitoringState> for Arc<trace::TraceReloadHandle> {
    fn from_ref(state: &MonitoringState) -> Self {
        state.trace_reload_handle.clone()
    }
}

#[tokio::main]
async fn main() {
    // Choose aws-lc-rs as the default rustls crypto provider. Specifying a
    // default provider here prevents runtime errors if another dependency also
    // enables the ring feature.
    let _ = aws_lc_rs::default_provider().install_default();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(-1);
        }
    };

    let (_guards, trace_reload_handle) = install_trace_subscriber(&config.trace_config()).unwrap();
    let cancel = CancellationToken::new();

    // Telemetry providers (OTLP push — endpoint via OTEL_EXPORTER_OTLP_ENDPOINT)
    let telemetry = telemetry::install_telemetry().expect("failed to install telemetry providers");

    // Monitoring server (traceconfig)
    let monitoring_state = MonitoringState {
        trace_reload_handle: Arc::new(trace_reload_handle),
    };
    let monitoring_router = axum::Router::new()
        .route(
            "/traceconfig",
            axum::routing::get(trace::get_traceconfig).put(trace::put_traceconfig),
        )
        .with_state(monitoring_state);
    let monitoring_listener = TcpListener::bind(config.monitoring_listen_address)
        .await
        .expect("failed to bind monitoring listener");
    let monitoring_cancel = cancel.clone();
    let monitoring_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(monitoring_listener, monitoring_router)
            .with_graceful_shutdown(monitoring_cancel.cancelled_owned())
            .await
        {
            tracing::error!("monitoring server error: {e}");
        }
    });

    // Main application
    let listen_address = config.listen_address;
    let app = build_app(config).await;

    let queue_handle = Queue::new(&app.db, &app.config, cancel.clone()).spawn_workers();

    let listener = TcpListener::bind(listen_address)
        .await
        .expect("failed to bind main listener");

    tracing::info!(
        "divviup-api {} listening on {listen_address}",
        env!("CARGO_PKG_VERSION")
    );

    let serve_result = axum::serve(listener, app.router)
        .with_graceful_shutdown(shutdown_signal(cancel.clone()))
        .await;
    // Ensure queue workers stop even if serve exits without a signal.
    cancel.cancel();

    if let Err(e) = serve_result {
        tracing::error!("server error: {e}");
    }

    if let Err(e) = queue_handle.await {
        tracing::error!("queue worker panic: {e}");
    }

    let _ = monitoring_handle.await;

    // Shut down telemetry providers last so in-flight spans and metrics from
    // the servers and queue workers are flushed before the OTLP exporters are
    // dropped. Traces are flushed before metrics.
    telemetry.shutdown();
}

async fn shutdown_signal(cancel: CancellationToken) {
    let ctrl_c = signal::ctrl_c();
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm.recv() => {},
    }
    tracing::info!("shutdown signal received, draining connections");
    cancel.cancel();
}
