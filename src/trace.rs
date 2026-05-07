//! Configures a tracing subscriber for divviup-api.

use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    io::{stdout, IsTerminal},
    net::SocketAddr,
    sync::Arc,
};
use tracing::Level;
use tracing_chrome::{ChromeLayerBuilder, TraceStyle};
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter::FromEnvError, layer::SubscriberExt, reload, EnvFilter, Layer, Registry,
};

/// Errors from initializing trace subscriber.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tracing error: {0}")]
    SetGlobalTracingSubscriber(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("logging error: {0}")]
    SetGlobalLogger(#[from] tracing_log::log_tracer::SetLoggerError),
    #[error("bad log/trace filter: {0}")]
    FromEnv(#[from] FromEnvError),
    #[error("{0}")]
    Other(&'static str),
}

/// Configuration for the tracing subscriber.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceConfig {
    /// If true, uses a [`tracing_subscriber::fmt::TestWriter`] to capture trace
    /// events when running tests
    #[serde(default)]
    pub use_test_writer: bool,
    /// If true OR if stdout is not a tty, trace events are output in JSON
    /// format by [`tracing_subscriber::fmt::format::Json`]. Otherwise, trace
    /// events are output in pretty format by
    /// [`tracing_subscriber::fmt::format::Pretty`].
    #[serde(default)]
    pub force_json_output: bool,
    /// If true, trace events are output in Google's Cloud Logging JSON format with
    /// [`tracing_stackdriver`].
    #[serde(default)]
    pub stackdriver_json_output: bool,
    /// Configuration for tokio-console monitoring and debugging support.
    /// (optional)
    #[serde(default)]
    pub tokio_console_config: TokioConsoleConfig,
    /// Flag to write tracing spans and events to JSON files. This is compatible with Chrome's
    /// trace viewer, available at `chrome://tracing`, and [Perfetto](https://ui.perfetto.dev).
    #[serde(default)]
    pub chrome: bool,
}

/// Configuration related to tokio-console.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokioConsoleConfig {
    /// If true, a tokio-console tracing subscriber is configured to monitor
    /// the async runtime, and listen for TCP connections. (Requires building
    /// with RUSTFLAGS="--cfg tokio_unstable")
    #[serde(default)]
    pub enabled: bool,
    /// Specifies an alternate address and port for the subscriber's gRPC
    /// server to listen on. If this is not present, it will use the value of
    /// the environment variable TOKIO_CONSOLE_BIND, or, failing that, a
    /// default of 127.0.0.1:6669.
    #[serde(default)]
    pub listen_address: Option<SocketAddr>,
}

/// Create a base tracing layer with configuration used in all subscribers
fn base_layer<S>() -> tracing_subscriber::fmt::Layer<S> {
    tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_level(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
}

/// Construct a filter to be used with tracing-opentelemetry and tracing-chrome, based on the
/// contents of the `RUST_TRACE` environment variable.
fn make_trace_filter() -> Result<EnvFilter, FromEnvError> {
    EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("RUST_TRACE")
        .from_env()
}

pub type TraceReloadHandle = reload::Handle<EnvFilter, Registry>;

/// Configures and installs a tracing subscriber, to capture events logged with [`tracing::info`]
/// and the like. Captured events are written to stdout, with formatting affected by the provided
/// [`TraceConfiguration`]. A handle to the stdout [`EnvFilter`] is provided, so that the filter
/// configuration can be altered later on at runtime.
pub fn install_trace_subscriber(
    config: &TraceConfig,
) -> Result<(TraceGuards, TraceReloadHandle), Error> {
    // If stdout is not a tty or if forced by config, output logs as JSON
    // structures
    let output_json = !stdout().is_terminal() || config.force_json_output;

    // Configure filters with RUST_LOG env var. Format discussed at
    // https://docs.rs/tracing-subscriber/latest/tracing_subscriber/struct.EnvFilter.html
    let (stdout_filter, stdout_filter_handle) =
        reload::Layer::new(EnvFilter::builder().from_env()?);

    let mut layers = Vec::new();
    match (
        output_json,
        config.use_test_writer,
        config.stackdriver_json_output,
    ) {
        (true, false, false) => layers.push(
            base_layer()
                .json()
                .with_current_span(false)
                .with_filter(stdout_filter)
                .boxed(),
        ),
        (false, false, false) => {
            layers.push(base_layer().pretty().with_filter(stdout_filter).boxed())
        }
        (_, true, false) => layers.push(
            base_layer()
                .pretty()
                .with_test_writer()
                .with_filter(stdout_filter)
                .boxed(),
        ),
        (_, _, true) => layers.push(
            tracing_stackdriver::layer()
                .with_filter(stdout_filter)
                .boxed(),
        ),
    }

    if config.tokio_console_config.enabled {
        let console_filter = tracing_subscriber::filter::Targets::new()
            .with_target("tokio", tracing::Level::TRACE)
            .with_target("runtime", tracing::Level::TRACE);

        let mut builder = console_subscriber::ConsoleLayer::builder();
        builder = builder.with_default_env();
        if let Some(listen_address) = &config.tokio_console_config.listen_address {
            builder = builder.server_addr(*listen_address);
        }
        layers.push(builder.spawn().with_filter(console_filter).boxed());
    }

    let mut chrome_guard = None;
    if config.chrome {
        let (layer, guard) = ChromeLayerBuilder::new()
            .trace_style(TraceStyle::Async)
            .include_args(true)
            .build();
        chrome_guard = Some(guard);
        layers.push(layer.with_filter(make_trace_filter()?).boxed());
    }

    let subscriber = Registry::default().with(layers);

    tracing::subscriber::set_global_default(subscriber)?;

    // Install a logger that converts logs into tracing events
    LogTracer::init()?;

    Ok((
        TraceGuards {
            _chrome_guard: chrome_guard,
        },
        stdout_filter_handle,
    ))
}

#[allow(missing_debug_implementations)]
pub struct TraceGuards {
    _chrome_guard: Option<tracing_chrome::FlushGuard>,
}

pub async fn get_traceconfig(
    State(handle): State<Arc<TraceReloadHandle>>,
) -> Result<String, (StatusCode, String)> {
    handle.with_current(|f| f.to_string()).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get current filter: {err}"),
        )
    })
}

/// Allows modifying the runtime tracing filter. Accepts a request whose body
/// is a valid tracing filter string. Responds with the updated filter. See
/// [`EnvFilter::try_new`] for details on the accepted format.
pub async fn put_traceconfig(
    State(handle): State<Arc<TraceReloadHandle>>,
    body: String,
) -> Result<String, (StatusCode, String)> {
    let new_filter = EnvFilter::try_new(body)
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("invalid filter: {err}")))?;
    handle.reload(new_filter).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to update filter: {err}"),
        )
    })?;
    handle.with_current(|f| f.to_string()).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get current filter: {err}"),
        )
    })
}
