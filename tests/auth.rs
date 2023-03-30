use divviup_api::USER_SESSION_KEY;
mod harness;
use harness::*;
use trillium_sessions::{Session, SessionConnExt};

mod login {
    use super::*;
    #[test]
    fn when_not_already_logged_in() {
        set_up(|app| async move {
            let conn = get("/login").run_async(&app).await;
            let auth_base = app.config().auth_url.join("/authorize")?;
            assert_status!(conn, 302);
            let location = conn
                .inner()
                .response_headers()
                .get_str(KnownHeaderName::Location)
                .unwrap();
            assert!(location.starts_with(auth_base.as_ref()));
            let url = Url::parse(location)?;
            let query = QueryStrong::parse(url.query().unwrap())?;
            assert_eq!(query["response_type"], "code");
            assert!(query.get_str("code_challenge").is_some());
            assert_eq!(query["client_id"], app.config().auth_client_id);
            assert_eq!(
                query["redirect_uri"],
                app.config().api_url.join("callback").unwrap().as_ref()
            );
            Ok(())
        });
    }

    #[test]
    fn when_already_logged_in() {
        set_up(|app| async move {
            let conn = get("/login").with_state(test_user()).run_async(&app).await;
            assert_response!(conn, 302, "", "Location" => app.config().app_url.as_ref());
            Ok(())
        });
    }
}

#[test]
fn logout() {
    set_up(|app| async move {
        let user = test_user();
        let mut session = Session::new();
        session.insert(USER_SESSION_KEY, &user)?;

        let conn = get("/logout").with_state(session).run_async(&app).await;

        assert!(conn.session().is_destroyed());

        assert_response!(conn, 302);
        let location: Url = conn
            .response_headers()
            .get_str(KnownHeaderName::Location)
            .unwrap()
            .parse()?;

        assert!(location
            .as_ref()
            .starts_with(app.config().auth_url.join("/v2/logout")?.as_ref()));

        let query = QueryStrong::parse(location.query().unwrap()).unwrap();
        assert_eq!(query["client_id"], app.config().auth_client_id);
        assert_eq!(query["returnTo"], app.config().app_url.as_ref());

        Ok(())
    });
}
