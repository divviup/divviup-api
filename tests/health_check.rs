mod harness;
use harness::{test, *};

#[test(harness = with_aggregator_api_mock)]
async fn health_check(app: DivviupApi) -> TestResult {
    assert_ok!(get("/health").with_api_host().run_async(&app).await);
    Ok(())
}
