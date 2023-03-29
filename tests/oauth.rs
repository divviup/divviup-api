use trillium_testing::prelude::*;
mod harness;
use harness::set_up;
use trillium::KnownHeaderName;

#[test]
fn when_not_already_logged_in() {
    set_up(|app| async move {
        let conn = get("/login").run_async(&app).await;
        let auth_base = app.config().auth_url.join("/authorize").unwrap();
        assert_status!(&conn, 302);
        let location = conn
            .inner()
            .response_headers()
            .get_str(KnownHeaderName::Location)
            .unwrap();
        assert!(location.starts_with(auth_base.as_ref()));
        let url = url::Url::parse(location).unwrap();
        let query = querystrong::QueryStrong::parse(url.query().unwrap()).unwrap();
        assert_eq!(query["response_type"], "code");
        assert!(query.get_str("code_challenge").is_some());
        assert_eq!(query["client_id"], app.config().auth_client_id);
        assert_eq!(
            query["redirect_uri"],
            app.config().api_url.join("callback").unwrap().as_ref()
        );
    });
}
