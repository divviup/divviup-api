use divviup_api::{
    api_mocks::aggregator_api::random_hpke_config, clients::aggregator_client::api_types,
};
use test_support::*;

mod index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::collector_credential(&app, &other_account).await;
        let (user, account, ..) = fixtures::member(&app).await;
        let collector_credential1 = fixtures::collector_credential(&app, &account).await;
        let collector_credential2 = fixtures::collector_credential(&app, &account).await;
        let deleted = fixtures::collector_credential(&app, &account).await;
        let _deleted = deleted.tombstone().update(app.db()).await.unwrap();

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_state(user)
        .run_async(&app)
        .await;
        assert_ok!(conn);

        let collector_credentials: Vec<CollectorCredential> = conn.response_json().await;
        assert_same_json_representation(
            &collector_credentials,
            &vec![collector_credential1, collector_credential2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::collector_credential(&app, &account).await;
        fixtures::collector_credential(&app, &account).await;

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
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
        fixtures::collector_credential(&app, &account).await;
        fixtures::collector_credential(&app, &account).await;

        let mut conn = get("/api/accounts/not-an-account/collector_credentials")
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
        let collector_credential1 = fixtures::collector_credential(&app, &account).await;
        let collector_credential2 = fixtures::collector_credential(&app, &account).await;

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_state(admin)
        .run_async(&app)
        .await;

        assert_ok!(conn);
        let collector_credentials: Vec<CollectorCredential> = conn.response_json().await;
        assert_same_json_representation(
            &collector_credentials,
            &vec![collector_credential1, collector_credential2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let collector_credential1 = fixtures::collector_credential(&app, &account).await;
        let collector_credential2 = fixtures::collector_credential(&app, &account).await;

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;

        assert_ok!(conn);
        let collector_credentials: Vec<CollectorCredential> = conn.response_json().await;
        assert_same_json_representation(
            &collector_credentials,
            &vec![collector_credential1, collector_credential2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let collector_credential1 = fixtures::collector_credential(&app, &account).await;
        let collector_credential2 = fixtures::collector_credential(&app, &account).await;

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;

        assert_ok!(conn);
        let collector_credentials: Vec<CollectorCredential> = conn.response_json().await;
        assert_same_json_representation(
            &collector_credentials,
            &vec![collector_credential1, collector_credential2],
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        fixtures::collector_credential(&app, &account).await;
        fixtures::collector_credential(&app, &account).await;

        let mut conn = get(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
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

    fn valid_collector_credential_json(hpke_config: api_types::HpkeConfig) -> Value {
        json!({
            "name": fixtures::random_name(),
            "contents": encode_hpke_config(hpke_config)
        })
    }

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let contents = random_hpke_config();

        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_state(user)
        .with_request_json(valid_collector_credential_json(contents.clone()))
        .run_async(&app)
        .await;
        assert_response!(conn, 201);
        let collector_credential: CollectorCredential = conn.response_json().await;
        let collector_credential_from_db = collector_credential.reload(app.db()).await?.unwrap();
        assert_eq!(collector_credential.contents, contents);
        assert_same_json_representation(&collector_credential, &collector_credential_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let collector_credential_count_before =
            CollectorCredentials::find().count(app.db()).await?;
        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_state(user)
        .with_request_json(valid_collector_credential_json(random_hpke_config()))
        .run_async(&app)
        .await;

        assert_not_found!(conn);
        let collector_credential_count_after = CollectorCredentials::find().count(app.db()).await?;
        assert_eq!(
            collector_credential_count_before,
            collector_credential_count_after
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let collector_credential_count_before =
            CollectorCredentials::find().count(app.db()).await?;

        let mut conn = post("/api/accounts/does-not-exist/collector_credentials")
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_collector_credential_json(random_hpke_config()))
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let collector_credential_count_after = CollectorCredentials::find().count(app.db()).await?;
        assert_eq!(
            collector_credential_count_before,
            collector_credential_count_after
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_request_json(valid_collector_credential_json(random_hpke_config()))
        .with_state(admin)
        .run_async(&app)
        .await;

        assert_response!(conn, 201);
        let collector_credential: CollectorCredential = conn.response_json().await;
        let collector_credential_from_db = collector_credential.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&collector_credential, &collector_credential_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .with_request_json(valid_collector_credential_json(random_hpke_config()))
        .run_async(&app)
        .await;

        assert_response!(conn, 201);
        let collector_credential: CollectorCredential = conn.response_json().await;
        let collector_credential_from_db = collector_credential.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&collector_credential, &collector_credential_from_db);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .with_request_json(valid_collector_credential_json(random_hpke_config()))
        .run_async(&app)
        .await;

        assert_response!(conn, 201);
        let collector_credential: CollectorCredential = conn.response_json().await;
        let collector_credential_from_db = collector_credential.reload(app.db()).await?.unwrap();
        assert_same_json_representation(&collector_credential, &collector_credential_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let count_before = CollectorCredentials::find().count(app.db()).await?;
        let mut conn = post(format!(
            "/api/accounts/{}/collector_credentials",
            account.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .with_request_json(valid_collector_credential_json(random_hpke_config()))
        .run_async(&app)
        .await;
        assert_not_found!(conn);
        let count_after = CollectorCredentials::find().count(app.db()).await?;
        assert_eq!(count_before, count_after);
        Ok(())
    }
}

mod delete {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_collector_credential(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete(format!("/api/collector_credentials/{}", Uuid::new_v4()))
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
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_state(user)
        .run_async(&app)
        .await;
        assert_status!(conn, 204);
        assert!(collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let mut conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_state(user)
        .run_async(&app)
        .await;
        assert_not_found!(conn);
        assert!(!collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_state(admin)
        .run_async(&app)
        .await;
        assert_status!(conn, 204);
        assert!(collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_status!(conn, 204);
        assert!(collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_status!(conn, 204);
        assert!(collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let mut conn = delete(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_not_found!(conn);
        assert!(!collector_credential
            .reload(app.db())
            .await?
            .unwrap()
            .is_tombstoned());
        Ok(())
    }
}

mod update {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_collector_credential(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = patch(format!("/api/collector_credentials/{}", Uuid::new_v4()))
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
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": name }))
        .with_state(user)
        .run_async(&app)
        .await;
        assert_status!(conn, 200);
        let response: CollectorCredential = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            collector_credential
                .reload(app.db())
                .await?
                .unwrap()
                .name
                .unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": fixtures::random_name() }))
        .with_state(user)
        .run_async(&app)
        .await;
        let name_before = collector_credential.name.clone();
        assert_not_found!(conn);
        assert_eq!(
            collector_credential.reload(app.db()).await?.unwrap().name,
            name_before
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": name }))
        .with_state(admin)
        .run_async(&app)
        .await;
        assert_status!(conn, 200);
        let response: CollectorCredential = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            collector_credential
                .reload(app.db())
                .await?
                .unwrap()
                .name
                .unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": name }))
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_status!(conn, 200);
        let response: CollectorCredential = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            collector_credential
                .reload(app.db())
                .await?
                .unwrap()
                .name
                .unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let name = fixtures::random_name();
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": name }))
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_status!(conn, 200);
        let response: CollectorCredential = conn.response_json().await;
        assert_eq!(response.name.unwrap(), name);
        assert_eq!(
            collector_credential
                .reload(app.db())
                .await?
                .unwrap()
                .name
                .unwrap(),
            name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let name_before = collector_credential.name.clone();
        let name = fixtures::random_name();
        let mut conn = patch(format!(
            "/api/collector_credentials/{}",
            collector_credential.id
        ))
        .with_api_headers()
        .with_request_json(json!({ "name": name }))
        .with_auth_header(token)
        .run_async(&app)
        .await;
        assert_not_found!(conn);
        assert_eq!(
            collector_credential.reload(app.db()).await?.unwrap().name,
            name_before
        );
        Ok(())
    }
}
