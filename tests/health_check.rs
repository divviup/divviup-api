mod harness;
use harness::{test, *};

#[test(harness = set_up)]
async fn health_check(app: DivviupApi) -> TestResult {
    assert_status!(get("/health").with_api_host().run_async(&app).await, 204);
    Ok(())
}
