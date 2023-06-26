mod harness;
use harness::*;

#[trillium::async_trait]
impl Reload for Aggregator {
    async fn reload(self, db: &impl ConnectionTrait) -> Result<Option<Self>, DbErr> {
        Aggregators::find_by_id(self.id).one(db).await
    }
}

#[track_caller]
fn assert_same_json_representation<T: serde::Serialize>(actual: &T, expected: &T) {
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

mod index {
    use super::{test, *};

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
}

mod create {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
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
        assert_eq!(aggregator.account_id.unwrap(), account.id);
        assert_eq!(
            aggregator.dap_url,
            Url::parse(&new_aggregator.dap_url.unwrap()).unwrap()
        );
        assert_eq!(
            &**aggregator.api_url.as_ref().unwrap(),
            &Url::parse(&new_aggregator.api_url.unwrap()).unwrap()
        );
        assert_eq!(aggregator.role.as_ref(), new_aggregator.role.unwrap());
        assert!(!aggregator.is_first_party);

        let aggregator_from_db = Aggregators::find_by_id(aggregator.id)
            .one(app.db())
            .await?
            .unwrap();

        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/aggregators", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(json!({
                "role": "whatever",
                "bearer_token": "not valid base64",
                "api_url": "not a url",
                "dap_url": "also not a url"
            }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let error: Value = conn.response_json().await;
        assert!(error.get("role").is_some());
        assert!(error.get("name").is_some());
        assert!(error.get("api_url").is_some());
        assert!(error.get("bearer_token").is_some());
        assert!(error.get("dap_url").is_some());
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

        let aggregator_from_db = Aggregators::find_by_id(aggregator.id)
            .one(app.db())
            .await?
            .unwrap();
        assert_same_json_representation(&aggregator, &aggregator_from_db);

        Ok(())
    }
}

mod show {
    use super::{test, *};

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
}

mod update {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn valid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let aggregator = fixtures::aggregator(&app, Some(&account)).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_aggregator: Aggregator = conn.response_json().await;
        assert_eq!(response_aggregator.name, new_name);
        assert_eq!(
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            new_name
        );

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
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
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
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            aggregator.name // unchanged
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn shared(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let aggregator = fixtures::aggregator(&app, None).await;

        let mut conn = patch(format!("/api/aggregators/{}", aggregator.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        assert_eq!(
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            aggregator.name // unchanged
        );
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
        assert_eq!(
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            new_name
        );

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
        assert_eq!(
            Aggregators::find_by_id(aggregator.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            new_name
        );

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
}

mod delete {
    use uuid::Uuid;

    use super::{test, *};

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
}

mod shared_create {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_admin(app: DivviupApi) -> TestResult {
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
        assert_eq!(
            aggregator.dap_url,
            Url::parse(&new_aggregator.dap_url.unwrap()).unwrap()
        );
        assert_eq!(
            &**aggregator.api_url.as_ref().unwrap(),
            &Url::parse(&new_aggregator.api_url.unwrap()).unwrap()
        );
        assert_eq!(aggregator.role.as_ref(), new_aggregator.role.unwrap());
        assert!(aggregator.account_id.is_none());
        assert!(aggregator.is_first_party);
        let aggregator_from_db = Aggregators::find_by_id(aggregator.id)
            .one(app.db())
            .await?
            .unwrap();

        assert_same_json_representation(&aggregator, &aggregator_from_db);

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
}
