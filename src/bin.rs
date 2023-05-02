use std::panic;

use divviup_api::{telemetry::install_metrics_exporter, ApiConfig, DivviupApi};
use trillium_http::Stopper;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = ApiConfig::from_env().expect("Missing config");
    let stopper = Stopper::new();

    let metrics_task_handle = install_metrics_exporter(
        &config.prometheus_host,
        config.prometheus_port,
        stopper.clone(),
    )
    .expect("Error setting up metrics");

    #[cfg(all(debug_assertions, feature = "aggregator-api-mock"))]
    if let Some(port) = config.aggregator_api_url.port() {
        tokio::task::spawn(
            trillium_tokio::config()
                .without_signals()
                .with_port(port)
                .with_stopper(stopper.clone())
                .run_async(divviup_api::aggregator_api_mock::aggregator_api()),
        );
    }
    let app = DivviupApi::new(config).await;

    let db = app.db().clone();
    let config = app.config().clone();
    tokio::task::spawn(async move {
        divviup_api::queue::run(db, config).await;
    });

    trillium_tokio::config()
        .with_stopper(stopper)
        .run_async(app)
        .await;

    if let Err(e) = metrics_task_handle.await {
        if let Ok(reason) = e.try_into_panic() {
            panic::resume_unwind(reason);
        }
    }
}
