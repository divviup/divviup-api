use test_support::{assert_eq, entity::session, test, *};

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

#[test(harness = set_up)]
async fn deleted_token_cannot_authenticate(app: DivviupApi) -> TestResult {
    let account = fixtures::account(&app).await;
    let (api_token, token) = fixtures::api_token(&app, &account).await;

    // Token works before deletion
    let conn = get("/api/accounts")
        .with_api_headers()
        .with_auth_header(token.clone())
        .run_async(&app)
        .await;
    assert_ok!(conn);

    // Delete (tombstone) the token
    api_token.tombstone().update(app.db()).await.unwrap();

    // Deleted token should no longer authenticate
    let conn = get("/api/accounts")
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;
    assert_status!(conn, 403);

    Ok(())
}

mod login {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn when_not_already_logged_in(app: DivviupApi) -> TestResult {
        let conn = get("/login").with_api_host().run_async(&app).await;
        let auth_base = app.config().auth_url.join("/authorize")?;
        assert_status!(conn, 302);
        let location = conn.header_str(headers::LOCATION.as_str()).unwrap();
        assert!(location.starts_with(auth_base.as_ref()));
        let url = Url::parse(location)?;
        let query = QueryStrong::parse_strict(url.query().unwrap()).unwrap();
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
            .with_user(&user)
            .run_async(&app)
            .await;
        assert_response!(conn, 302, "", headers::LOCATION => app.config().app_url.as_ref());
        Ok(())
    }
}

#[test(harness = set_up)]
async fn logout(app: DivviupApi) -> TestResult {
    let user = fixtures::user();
    let conn = get("/logout")
        .with_api_host()
        .with_user(&user)
        .run_async(&app)
        .await;

    assert_response!(conn, 302);

    // session.flush() in the logout handler should delete any session that was
    // created during the request — verify no sessions remain in the store.
    let session_count = session::Entity::find().count(app.db()).await?;
    assert_eq!(session_count, 0, "session should be destroyed after logout");
    let location: Url = conn
        .header_str(headers::LOCATION.as_str())
        .unwrap()
        .parse()?;

    assert!(location
        .as_ref()
        .starts_with(app.config().auth_url.join("/v2/logout")?.as_ref()));

    let query = QueryStrong::parse_strict(location.query().unwrap()).unwrap();
    assert_eq!(query["client_id"], app.config().auth_client_id);
    assert_eq!(query["returnTo"], app.config().app_url.as_ref());

    Ok(())
}
