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

/// Verify that `/health` — the first real route migrated to Axum — is
/// reached via the Trillium → proxy → Axum path.
#[test(harness = set_up)]
async fn axum_proxies_health(app: DivviupApi) -> TestResult {
    let conn = get("/health").with_api_host().run_async(&app).await;
    assert_eq!(conn.status().unwrap(), Status::Ok);
    Ok(())
}
