use divviup_api::{ApiConfig, DivviupApi, Queue};

use trillium_http::Stopper;
use trillium_tokio::CloneCounterObserver;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match ApiConfig::from_env() {
        Ok(config) => config,
        Err(e) => panic!("{e}"),
    };
    let stopper = Stopper::new();
    let observer = CloneCounterObserver::default();

    trillium_tokio::config()
        .without_signals()
        .with_port(config.prometheus_port)
        .with_host(&config.prometheus_host)
        .with_observer(observer.clone())
        .with_stopper(stopper.clone())
        .spawn(divviup_api::telemetry::metrics_exporter().unwrap());

    #[cfg(all(debug_assertions, feature = "aggregator-api-mock"))]
    if let Some(port) = config.aggregator_api_url.port() {
        trillium_tokio::config()
            .without_signals()
            .with_port(port)
            .with_observer(observer.clone())
            .with_stopper(stopper.clone())
            .spawn(divviup_api::aggregator_api_mock::aggregator_api());
    }

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
