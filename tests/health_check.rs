use trillium_testing::prelude::*;
mod harness;
#[test]
fn root() {
    harness::with_server(|app| async move {
        assert_ok!(get("/").run_async(&app).await);
    });
}

#[test]
fn health_check() {
    harness::with_server(|app| async move {
        assert_body!(get("/health").run_async(&app).await, "");
    });
}
