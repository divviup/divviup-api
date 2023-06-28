#![cfg(assets)]
use test_support::{assert_eq, test, *};

const INDEX_FILE_SNIPPET: &str = "<!DOCTYPE html>";

#[test(harness = set_up)]
async fn root_serves_index(app: DivviupApi) -> TestResult {
    assert_body_contains!(
        get("/").with_app_host().run_async(&app).await,
        INDEX_FILE_SNIPPET
    );
    Ok(())
}

#[test(harness = set_up)]
async fn not_found_path_serves_index(app: DivviupApi) -> TestResult {
    assert_body_contains!(
        get("/this-is/some-arbitrary-path?and-it&does-not-matter")
            .with_app_host()
            .run_async(&app)
            .await,
        INDEX_FILE_SNIPPET
    );
    Ok(())
}

#[test(harness = set_up)]
async fn api_path_serves_index(app: DivviupApi) -> TestResult {
    assert_body_contains!(
        get("/api/users/me").with_app_host().run_async(&app).await,
        INDEX_FILE_SNIPPET
    );
    Ok(())
}

#[test(harness = set_up)]
async fn api_url(app: DivviupApi) -> TestResult {
    assert_ok!(
        get("/api_url").with_app_host().run_async(&app).await,
        "https://api.example/",
        "cache-control" => "no-cache",
        "content-type" => "text/plain"
    );
    Ok(())
}

#[test(harness = set_up)]
async fn static_files(app: DivviupApi) -> TestResult {
    let mut html_conn = get("/").with_app_host().run_async(&app).await;

    assert_ok!(&html_conn);
    assert_headers!(&html_conn, "cache-control" => "no-cache");

    let html = html_conn.take_response_body_string().unwrap();

    let regex = regex::Regex::new(r#"script type="module" crossorigin src="([^"]+)""#).unwrap();
    let js_path = &regex.captures_iter(&html).next().unwrap()[1];
    let js_conn = get(js_path).with_app_host().run_async(&app).await;
    assert_ok!(&js_conn);
    assert_headers!(&js_conn, "cache-control" => "max-age=31536000");

    Ok(())
}
