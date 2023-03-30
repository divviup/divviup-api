mod harness;
use harness::*;

#[test]
fn root() {
    set_up(|app| async move {
        assert_ok!(get("/").run_async(&app).await);
        Ok(())
    });
}

#[test]
fn health_check() {
    set_up(|app| async move {
        assert_ok!(get("/health").run_async(&app).await);
        Ok(())
    });
}
