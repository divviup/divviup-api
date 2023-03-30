use trillium_testing::prelude::*;
mod harness;
use harness::set_up;

#[test]
fn root() {
    set_up(|app| async move {
        assert_ok!(get("/").run_async(&app).await);
    });
}

#[test]
fn health_check() {
    set_up(|app| async move {
        assert_ok!(get("/health").run_async(&app).await);
    });
}
