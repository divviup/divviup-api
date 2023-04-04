use divviup_api::{telemetry::install_metrics_exporter, ApiConfig, DivviupApi};
use trillium_http::Stopper;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = ApiConfig::from_env().expect("Missing config");
    let stopper = Stopper::new();

    install_metrics_exporter(&config.prometheus_host, config.prometheus_port)
        .expect("Error setting up metrics");

    #[cfg(all(debug_assertions, feature = "aggregator-api-mock"))]
    if let Some(port) = config.aggregator_url.port() {
        tokio::task::spawn(
            trillium_tokio::config()
                .without_signals()
                .with_port(port)
                .with_stopper(stopper.clone())
                .run_async(divviup_api::aggregator_api_mock::aggregator_api()),
        );
    }

    trillium_tokio::config()
        .with_stopper(stopper)
        .run_async(DivviupApi::new(config).await)
        .await;
}
