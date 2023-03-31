mod harness;
use harness::{test, *};

#[test(harness = set_up)]
async fn root(app: DivviupApi) -> TestResult {
    assert_ok!(get("/").run_async(&app).await);
    Ok(())
}

#[test(harness = set_up)]
async fn health_check(app: DivviupApi) -> TestResult {
    assert_ok!(get("/health").run_async(&app).await);
    Ok(())
}
