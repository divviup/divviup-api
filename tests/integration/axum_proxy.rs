use test_support::{assert_eq, test, *};

#[test(harness = set_up)]
async fn axum_proxy_bridge(app: DivviupApi) -> TestResult {
    let mut conn = get("/internal/test/axum_ready")
        .with_api_host()
        .run_async(&app)
        .await;
    assert_eq!(conn.status().unwrap(), Status::Ok);
    assert_eq!(conn.take_response_body_string().unwrap(), "axum OK");
    Ok(())
}

/// Verify that a route handled by Trillium (health check) is served
/// directly and not double-forwarded through the proxy to Axum.
#[test(harness = set_up)]
async fn trillium_route_not_proxied(app: DivviupApi) -> TestResult {
    let conn = get("/health").with_api_host().run_async(&app).await;
    assert_eq!(conn.status().unwrap(), Status::Ok);
    Ok(())
}
