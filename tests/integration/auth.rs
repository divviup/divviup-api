use divviup_api::USER_SESSION_KEY;
use test_support::{assert_eq, test, *};
use trillium_sessions::{Session, SessionConnExt};

#[test(harness = set_up)]
async fn first_use_of_a_token_updates_last_used_at(app: DivviupApi) -> TestResult {
    let account = fixtures::account(&app).await;
    let (api_token, token) = fixtures::api_token(&app, &account).await;
    assert!(api_token.last_used_at.is_none());
    get("/api/accounts") //this could be any token-authorized route
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;
    assert!(api_token
        .reload(app.db())
        .await?
        .unwrap()
        .last_used_at
        .is_some());
    Ok(())
}

mod login {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn when_not_already_logged_in(app: DivviupApi) -> TestResult {
        let conn = get("/login").with_api_host().run_async(&app).await;
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
    }

    #[test(harness = set_up)]
    async fn when_already_logged_in(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let conn = get("/login")
            .with_api_host()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 302, "", "Location" => app.config().app_url.as_ref());
        Ok(())
    }
}

#[test(harness = set_up)]
async fn logout(app: DivviupApi) -> TestResult {
    let user = fixtures::user();
    let mut session = Session::new();
    session.insert(USER_SESSION_KEY, &user)?;

    let conn = get("/logout")
        .with_api_host()
        .with_state(session)
        .run_async(&app)
        .await;

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
}
