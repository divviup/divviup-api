use test_support::{assert_eq, test, *};

#[test(harness = set_up)]
async fn health_check(app: DivviupApi) -> TestResult {
    assert_ok!(get("/health").with_api_host().run_async(&app).await);
    Ok(())
}
