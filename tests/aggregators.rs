use divviup_api::{clients::aggregator_client::AggregatorApiConfig, entity::aggregator::Role};
use std::str::FromStr;
use test_support::{assert_eq, *};

mod index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::aggregator(&app, Some(&other_account)).await;
        let shared_aggregator = fixtures::aggregator(&app, None).await;

        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator, aggregator1, aggregator2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn does_not_include_tombstoned_aggregators(app: DivviupApi) -> TestResult {
        let shared_aggregator = fixtures::aggregator(&app, None).await;
        shared_aggregator.tombstone().update(app.db()).await?;

        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        aggregator1.tombstone().update(app.db()).await?;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(&aggregators, &vec![aggregator2]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn does_not_include_tombstoned_aggregators_as_admin(app: DivviupApi) -> TestResult {
        let shared_aggregator = fixtures::aggregator(&app, None).await;
        shared_aggregator.tombstone().update(app.db()).await?;

        let (admin, account, ..) = fixtures::admin(&app).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        aggregator1.tombstone().update(app.db()).await?;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(&vec![aggregator2], &aggregators);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        fixtures::aggregator(&app, None).await;
        let account = fixtures::account(&app).await;
        fixtures::aggregator(&app, Some(&account)).await;
        fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
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
        fixtures::aggregator(&app, None).await;

        let account = fixtures::account(&app).await;
        fixtures::aggregator(&app, Some(&account)).await;
        fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get("/api/accounts/not-an-account/aggregators")
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
        let shared_aggregator = fixtures::aggregator(&app, None).await;

        let account = fixtures::account(&app).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator, aggregator1, aggregator2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::aggregator(&app, Some(&other_account)).await;
        let shared_aggregator = fixtures::aggregator(&app, None).await;

        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator, aggregator1, aggregator2],
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::aggregator(&app, Some(&other_account)).await;
        let shared_aggregator = fixtures::aggregator(&app, None).await;

        let account = fixtures::account(&app).await;
        let aggregator1 = fixtures::aggregator(&app, Some(&account)).await;
        let aggregator2 = fixtures::aggregator(&app, Some(&account)).await;

        let admin_token = fixtures::admin_token(&app).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, admin_token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator, aggregator1, aggregator2],
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        fixtures::aggregator(&app, Some(&other_account)).await;
        fixtures::aggregator(&app, None).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        fixtures::aggregator(&app, Some(&account)).await;
        fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_request_header(KnownHeaderName::Authorization, token)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }
}

mod shared_aggregator_index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_user(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::aggregator(&app, Some(&other_account)).await;

        let shared_aggregator1 = fixtures::aggregator(&app, None).await;
        let shared_aggregator2 = fixtures::aggregator(&app, None).await;

        let (user, account, ..) = fixtures::member(&app).await;
        fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get("/api/aggregators")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator1, shared_aggregator2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::aggregator(&app, Some(&other_account)).await;

        let shared_aggregator1 = fixtures::aggregator(&app, None).await;
        let shared_aggregator2 = fixtures::aggregator(&app, None).await;

        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = get("/api/aggregators")
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let aggregators: Vec<Aggregator> = conn.response_json().await;
        assert_same_json_representation(
            &aggregators,
            &vec![shared_aggregator1, shared_aggregator2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_logged_in(app: DivviupApi) -> TestResult {
        let conn = get("/api/aggregators")
            .with_api_headers()
            .run_async(&app)
            .await;

        assert_status!(conn, 403);
        Ok(())
    }
}

mod create {
    use divviup_api::api_mocks::aggregator_api::BAD_BEARER_TOKEN;

    use super::{assert_eq, test, *};

    #[test(harness = with_client_logs)]
    async fn success(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let new_aggregator = fixtures::new_aggregator();
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(new_aggregator.clone())
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;

        let aggregator_config: AggregatorApiConfig = client_logs.last().response_json();

        assert_eq!(aggregator.account_id.unwrap(), account.id);
        assert_eq!(aggregator.dap_url, aggregator_config.dap_url);
        assert_eq!(
            aggregator.api_url,
            Url::parse(&new_aggregator.api_url.unwrap()).unwrap()
        );
        assert_eq!(
            Role::from_str(aggregator.role.as_ref()).unwrap(),
            aggregator_config.role
        );
        assert!(!aggregator.is_first_party);

        let aggregator_from_db = aggregator.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn is_first_party_is_ignored(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut new_aggregator = fixtures::new_aggregator();
        new_aggregator.is_first_party = Some(true);
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(new_aggregator)
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(!aggregator.is_first_party);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_first_party);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(json!({
                "bearer_token": "not valid base64",
                "api_url": "not a url",
            }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let error: Value = conn.response_json().await;
        assert!(error.get("name").is_some());
        assert!(error.get("api_url").is_some());
        assert!(error.get("bearer_token").is_some());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let aggregator_count_before = Aggregators::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;

        let mut conn = post("/api/accounts/does-not-exist/aggregators")
            .with_api_headers()
            .with_state(user)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(admin)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(!aggregator.is_first_party);

        let aggregator_from_db = aggregator.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(!aggregator.is_first_party);

        let aggregator_from_db = aggregator.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(!aggregator.is_first_party);

        let aggregator_from_db = aggregator.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&aggregator, &aggregator_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(fixtures::new_aggregator())
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);

        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn aggregator_api_forbidden(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut new_aggregator = fixtures::new_aggregator();
        new_aggregator.bearer_token = Some(BAD_BEARER_TOKEN.to_string());
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;

        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(new_aggregator.clone())
            .run_async(&app)
            .await;
        assert_response!(conn, 400);
        assert_eq!(client_logs.last().response_status, Status::Unauthorized);
        let error: Value = conn.response_json().await;
        assert!(error.get("bearer_token").is_some());
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);

        Ok(())
    }
}

mod show {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_same_json_representation(&response_aggregator, &aggregator);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared_aggregator(app: DivviupApi) -> TestResult {
        let (user, _account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_same_json_representation(&response_aggregator, &aggregator);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
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
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_same_json_representation(&response_aggregator, &aggregator);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_aggregator(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = get("/api/aggregators/some-made-up-id")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account))
            .await
            .tombstone()
            .update(app.db())
            .await?;

        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_shared(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, None)
            .await
            .tombstone()
            .update(app.db())
            .await?;

        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account))
            .await
            .tombstone()
            .update(app.db())
            .await?;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_shared_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let aggregator = fixtures::aggregator(&app, None)
            .await
            .tombstone()
            .update(app.db())
            .await?;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response: Aggregator = conn.response_json().await;
        assert_same_json_representation(&aggregator, &response);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response: Aggregator = conn.response_json().await;
        assert_same_json_representation(&aggregator, &response);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = get(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }
}

mod update {
    use divviup_api::api_mocks::aggregator_api::BAD_BEARER_TOKEN;

    use super::{assert_eq, test, *};

    #[test(harness = with_client_logs)]
    async fn valid(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let new_bearer_token = fixtures::random_name();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name, "bearer_token": &new_bearer_token }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        assert_eq!(client_logs.logs().len(), 1);
        assert_eq!(
            client_logs
                .last()
                .request_headers
                .get_str(KnownHeaderName::Authorization)
                .unwrap(),
            format!("Bearer {new_bearer_token}")
        );
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, new_name);
        let reloaded = aggregator.reload(app.db()).await?.unwrap();
        assert_eq!(reloaded.name, new_name);
        assert_eq!(reloaded.bearer_token, new_bearer_token);

        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn bad_bearer_token(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let original_bearer_token = aggregator.bearer_token.clone();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "bearer_token": &BAD_BEARER_TOKEN }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 400);
        assert_eq!(client_logs.logs().len(), 1);
        assert_eq!(
            client_logs
                .last()
                .request_headers
                .get_str(KnownHeaderName::Authorization)
                .unwrap(),
            format!("Bearer {BAD_BEARER_TOKEN}")
        );

        assert_eq!(
            aggregator.reload(app.db()).await?.unwrap().bearer_token,
            original_bearer_token
        );
        assert_eq!(client_logs.last().response_status, Status::Unauthorized);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("bearer_token").is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());

        assert_eq!(
            aggregator.reload(app.db()).await?.unwrap().name,
            aggregator.name // unchanged
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        assert_eq!(
            aggregator.reload(app.db()).await?.unwrap().name,
            aggregator.name // unchanged
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let aggregator = fixtures::aggregator(&app, None).await;
        let old_name = aggregator.name.clone();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        assert_eq!(aggregator.reload(app.db()).await?.unwrap().name, old_name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let new_name = format!("new name {}", fixtures::random_name());

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, new_name);
        assert_eq!(aggregator.reload(app.db()).await?.unwrap().name, new_name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, new_name);
        assert_eq!(aggregator.reload(app.db()).await?.unwrap().name, new_name);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_aggregator(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = patch("/api/aggregators/not-an-aggregator-id")
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account))
            .await
            .tombstone()
            .update(app.db())
            .await?;

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new_name" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_shared(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, None)
            .await
            .tombstone()
            .update(app.db())
            .await?;

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new_name" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account))
            .await
            .tombstone()
            .update(app.db())
            .await?;
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new_name" }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn tombstoned_shared_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let aggregator = fixtures::aggregator(&app, None)
            .await
            .tombstone()
            .update(app.db())
            .await?;
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new_name" }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(json!({ "name": name }))
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, name);
        assert_eq!(aggregator.reload(app.db()).await?.unwrap().name, name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(json!({ "name": name }))
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, name);
        assert_eq!(aggregator.reload(app.db()).await?.unwrap().name, name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let name_before = aggregator.name.clone();
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(json!({ "name": fixtures::random_name() }))
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert_eq!(
            aggregator.reload(app.db()).await?.unwrap().name,
            name_before
        );

        Ok(())
    }
}

mod delete {
    use super::{assert_eq, test, *};
    use uuid::Uuid;

    #[test(harness = set_up)]
    #[ignore]
    async fn nonexistant_aggregator(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete(format!("/api/aggregators/{}", Uuid::new_v4()))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared_as_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let mut conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;
        let mut conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonadmin_token_shared_aggregator(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let mut conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token_shared_aggregator(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let aggregator = fixtures::aggregator(&app, None).await;
        let conn = delete(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }
}

mod shared_create {
    use super::{assert_eq, test, *};

    #[test(harness = with_client_logs)]
    async fn as_admin(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let new_aggregator = fixtures::new_aggregator();
        let mut conn = post("/api/aggregators")
            .with_request_json(&new_aggregator)
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        let aggregator_config: AggregatorApiConfig = client_logs.last().response_json();

        assert!(aggregator.account_id.is_none());
        assert_eq!(aggregator.dap_url, aggregator_config.dap_url);
        assert_eq!(
            aggregator.api_url,
            Url::parse(&new_aggregator.api_url.unwrap()).unwrap()
        );
        assert_eq!(
            Role::from_str(aggregator.role.as_ref()).unwrap(),
            aggregator_config.role
        );
        // defaults to true when not specified
        assert!(aggregator.is_first_party);

        let aggregator_from_db = aggregator.reload(app.db()).await?.unwrap();
        assert!(aggregator_from_db.is_first_party);
        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn is_first_party_true_is_accepted(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let mut new_aggregator = fixtures::new_aggregator();
        new_aggregator.is_first_party = Some(true);
        let mut conn = post("/api/aggregators")
            .with_api_headers()
            .with_state(admin)
            .with_request_json(new_aggregator)
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(aggregator.is_first_party);
        assert!(aggregator.reload(app.db()).await?.unwrap().is_first_party);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn is_first_party_false_is_accepted(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let mut new_aggregator = fixtures::new_aggregator();
        new_aggregator.is_first_party = Some(false);
        let mut conn = post("/api/aggregators")
            .with_api_headers()
            .with_state(admin)
            .with_request_json(new_aggregator)
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let aggregator: Aggregator = conn.response_json().await;
        assert!(!aggregator.is_first_party);
        assert!(!aggregator.reload(app.db()).await?.unwrap().is_first_party);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_nonadmin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::member(&app).await;
        let new_aggregator = fixtures::new_aggregator();
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;

        let mut conn = post("/api/aggregators")
            .with_request_json(&new_aggregator)
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonadmin_token_shared_aggregator(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let new_aggregator = fixtures::new_aggregator();
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;

        let mut conn = post("/api/aggregators")
            .with_request_json(&new_aggregator)
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_before, aggregator_count_after);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token_shared_aggregator(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let new_aggregator = fixtures::new_aggregator();
        let aggregator_count_before = Aggregators::find().count(app.db()).await?;
        let mut conn = post("/api/aggregators")
            .with_request_json(&new_aggregator)
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        let aggregator: Aggregator = conn.response_json().await;
        assert_eq!(aggregator.name, new_aggregator.name.unwrap());
        let aggregator_count_after = Aggregators::find().count(app.db()).await?;
        assert_eq!(aggregator_count_after, aggregator_count_before + 1);

        Ok(())
    }
}
