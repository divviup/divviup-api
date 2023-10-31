use divviup_api::{
    trace::{install_trace_subscriber, traceconfig_handler},
    Config, DivviupApi, Queue,
};
use trillium_http::Stopper;
use trillium_tokio::CloneCounterObserver;

#[tokio::main]
async fn main() {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(-1);
        }
    };

    let (_guards, trace_reload_handle) = install_trace_subscriber(&config.trace_config()).unwrap();

    let stopper = Stopper::new();
    let observer = CloneCounterObserver::default();

    trillium_tokio::config()
        .without_signals()
        .with_socketaddr(config.monitoring_listen_address)
        .with_observer(observer.clone())
        .with_stopper(stopper.clone())
        .spawn((
            divviup_api::telemetry::metrics_exporter().unwrap(),
            traceconfig_handler(trace_reload_handle),
        ));

    let app = DivviupApi::new(config).await;

    Queue::new(app.db(), app.config())
        .with_observer(observer.clone())
        .with_stopper(stopper.clone())
        .spawn_workers();

    trillium_tokio::config()
        .with_stopper(stopper)
        .with_observer(observer)
        .spawn(app)
        .await;
}
