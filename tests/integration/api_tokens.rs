use test_support::*;

mod index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::api_token(&app, &other_account).await;
        let (user, account, ..) = fixtures::member(&app).await;
        let (token1, _) = fixtures::api_token(&app, &account).await;
        let (token2, _) = fixtures::api_token(&app, &account).await;
        let (deleted, _) = fixtures::api_token(&app, &account).await;
        let _deleted = deleted.tombstone().update(app.db()).await.unwrap();

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);

        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(&api_tokens, &vec![token2, token1]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::api_token(&app, &account).await;
        fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::api_token(&app, &account).await;
        fixtures::api_token(&app, &account).await;

        let mut conn = get("/api/accounts/not-an-account/api_tokens")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token1, _) = fixtures::api_token(&app, &account).await;
        let (api_token2, _) = fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(&api_tokens, &vec![api_token2, api_token1]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token1, _) = fixtures::api_token(&app, &account).await;
        let (api_token2, _) = fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(&api_tokens, &vec![api_token2, api_token1]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (api_token1, token) = fixtures::api_token(&app, &account).await;
        let (api_token2, _) = fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(
            &api_tokens,
            &vec![api_token2, api_token1.reload(app.db()).await?.unwrap()],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        fixtures::api_token(&app, &account).await;
        fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }
}

mod create {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let mut api_token: ApiToken = conn.response_json().await;
        let api_token_from_db = api_token.reload(app.db()).await?.unwrap();
        let (api_token_from_token, account_from_token) =
            ApiTokens::load_and_check(&api_token.token.take().unwrap(), app.db())
                .await
                .unwrap();

        assert_eq!(api_token, api_token_from_token);
        assert_eq!(account, account_from_token);
        assert_same_json_representation(&api_token, &api_token_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let api_token_count_before = ApiTokens::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let api_token_count_after = ApiTokens::find().count(app.db()).await?;
        assert_eq!(api_token_count_before, api_token_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let api_token_count_before = ApiTokens::find().count(app.db()).await?;

        let mut conn = post("/api/accounts/does-not-exist/api_tokens")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let api_token_count_after = ApiTokens::find().count(app.db()).await?;
        assert_eq!(api_token_count_before, api_token_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let mut api_token: ApiToken = conn.response_json().await;
        let api_token_from_db = api_token.reload(app.db()).await?.unwrap();
        let (api_token_from_token, account_from_token) =
            ApiTokens::load_and_check(&api_token.token.take().unwrap(), app.db())
                .await
                .unwrap();

        assert_eq!(api_token, api_token_from_token);
        assert_eq!(account, account_from_token);
        assert_same_json_representation(&api_token, &api_token_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let mut api_token: ApiToken = conn.response_json().await;
        let api_token_from_db = api_token.reload(app.db()).await?.unwrap();
        let (api_token_from_token, account_from_token) =
            ApiTokens::load_and_check(&api_token.token.take().unwrap(), app.db())
                .await
                .unwrap();

        assert_eq!(api_token, api_token_from_token);
        assert_eq!(account, account_from_token);
        assert_same_json_representation(&api_token, &api_token_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let mut api_token: ApiToken = conn.response_json().await;
        let api_token_from_db = api_token.reload(app.db()).await?.unwrap();
        let (api_token_from_token, account_from_token) =
            ApiTokens::load_and_check(&api_token.token.take().unwrap(), app.db())
                .await
                .unwrap();

        assert_eq!(api_token, api_token_from_token);
        assert_eq!(account, account_from_token);
        assert_same_json_representation(&api_token, &api_token_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let count_before = ApiTokens::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        let count_after = ApiTokens::find().count(app.db()).await?;
        assert_eq!(count_before, count_after);
        Ok(())
    }
}

mod delete {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_api_token(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete(format!("/api/api_tokens/{}", Uuid::new_v4()))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let (api_token, ..) = fixtures::api_token(&app, &account).await;
        let mut conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let mut conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!api_token.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }
}

mod update {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_api_token(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = patch(format!("/api/api_tokens/{}", Uuid::new_v4()))
            .with_request_json(json!({ "name": fixtures::random_name() }))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: ApiToken = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            api_token.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let (api_token, ..) = fixtures::api_token(&app, &account).await;
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": fixtures::random_name() }))
            .with_state(user)
            .run_async(&app)
            .await;
        let name_before = api_token.name.clone();
        assert_not_found!(conn);
        assert_eq!(api_token.reload(app.db()).await?.unwrap().name, name_before);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: ApiToken = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            api_token.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: ApiToken = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            api_token.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (api_token, token) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: ApiToken = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            api_token.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(api_token.reload(app.db()).await?.unwrap().name.is_none());
        Ok(())
    }
}
