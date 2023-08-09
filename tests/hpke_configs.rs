use divviup_api::{
    api_mocks::aggregator_api::random_hpke_config, clients::aggregator_client::api_types,
};
use test_support::*;

mod index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::hpke_config(&app, &other_account).await;
        let (user, account, ..) = fixtures::member(&app).await;
        let hpke_config1 = fixtures::hpke_config(&app, &account).await;
        let hpke_config2 = fixtures::hpke_config(&app, &account).await;
        let deleted = fixtures::hpke_config(&app, &account).await;
        let _deleted = deleted.tombstone().update(app.db()).await.unwrap();

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);

        let hpke_configs: Vec<HpkeConfig> = conn.response_json().await;
        assert_same_json_representation(&hpke_configs, &vec![hpke_config1, hpke_config2]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::hpke_config(&app, &account).await;
        fixtures::hpke_config(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
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
        fixtures::hpke_config(&app, &account).await;
        fixtures::hpke_config(&app, &account).await;

        let mut conn = get("/api/accounts/not-an-account/hpke_configs")
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
        let hpke_config1 = fixtures::hpke_config(&app, &account).await;
        let hpke_config2 = fixtures::hpke_config(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let hpke_configs: Vec<HpkeConfig> = conn.response_json().await;
        assert_same_json_representation(&hpke_configs, &vec![hpke_config1, hpke_config2]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let hpke_config1 = fixtures::hpke_config(&app, &account).await;
        let hpke_config2 = fixtures::hpke_config(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let hpke_configs: Vec<HpkeConfig> = conn.response_json().await;
        assert_same_json_representation(&hpke_configs, &vec![hpke_config1, hpke_config2]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let hpke_config1 = fixtures::hpke_config(&app, &account).await;
        let hpke_config2 = fixtures::hpke_config(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let hpke_configs: Vec<HpkeConfig> = conn.response_json().await;
        assert_same_json_representation(&hpke_configs, &vec![hpke_config1, hpke_config2]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        fixtures::hpke_config(&app, &account).await;
        fixtures::hpke_config(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/hpke_configs", account.id))
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

    fn valid_hpke_config_json(hpke_config: api_types::HpkeConfig) -> Value {
        json!({
            "name": fixtures::random_name(),
            "contents": encode_hpke_config(hpke_config)
        })
    }

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let contents = random_hpke_config();

        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_hpke_config_json(contents.clone()))
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let hpke_config: HpkeConfig = conn.response_json().await;
        let hpke_config_from_db = hpke_config.reload(app.db()).await?.unwrap();
        assert_eq!(hpke_config.contents, contents);
        assert_same_json_representation(&hpke_config, &hpke_config_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let hpke_config_count_before = HpkeConfigs::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let hpke_config_count_after = HpkeConfigs::find().count(app.db()).await?;
        assert_eq!(hpke_config_count_before, hpke_config_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let hpke_config_count_before = HpkeConfigs::find().count(app.db()).await?;

        let mut conn = post("/api/accounts/does-not-exist/hpke_configs")
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let hpke_config_count_after = HpkeConfigs::find().count(app.db()).await?;
        assert_eq!(hpke_config_count_before, hpke_config_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let hpke_config: HpkeConfig = conn.response_json().await;
        let hpke_config_from_db = hpke_config.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&hpke_config, &hpke_config_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let hpke_config: HpkeConfig = conn.response_json().await;
        let hpke_config_from_db = hpke_config.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&hpke_config, &hpke_config_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let hpke_config: HpkeConfig = conn.response_json().await;
        let hpke_config_from_db = hpke_config.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&hpke_config, &hpke_config_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let count_before = HpkeConfigs::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/hpke_configs", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_hpke_config_json(random_hpke_config()))
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        let count_after = HpkeConfigs::find().count(app.db()).await?;
        assert_eq!(count_before, count_after);
        Ok(())
    }
}

mod delete {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_hpke_config(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete(format!("/api/hpke_configs/{}", Uuid::new_v4()))
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
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let mut conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let mut conn = delete(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!hpke_config.reload(app.db()).await?.unwrap().is_tombstoned());
        Ok(())
    }
}

mod update {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_hpke_config(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = patch(format!("/api/hpke_configs/{}", Uuid::new_v4()))
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
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: HpkeConfig = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": fixtures::random_name() }))
            .with_state(user)
            .run_async(&app)
            .await;
        let name_before = hpke_config.name.clone();
        assert_not_found!(conn);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name,
            name_before
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: HpkeConfig = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: HpkeConfig = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_status!(conn, 200);
        let response: HpkeConfig = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name.unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let hpke_config = fixtures::hpke_config(&app, &account).await;
        let name_before = hpke_config.name.clone();
        let name = fixtures::random_name();
        let mut conn = patch(format!("/api/hpke_configs/{}", hpke_config.id))
            .with_api_headers()
            .with_request_json(json!({ "name": name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert_eq!(
            hpke_config.reload(app.db()).await?.unwrap().name,
            name_before
        );
        Ok(())
    }
}
