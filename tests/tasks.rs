mod harness;
use harness::*;

mod index {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::task(&app, &other_account).await;

        let (user, account, ..) = fixtures::member(&app).await;
        let task1 = fixtures::task(&app, &account).await;
        let task2 = fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let tasks: Vec<Task> = conn.response_json().await;
        assert_eq!(vec![task1, task2], tasks);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::task(&app, &account).await;
        fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
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
        fixtures::task(&app, &account).await;
        fixtures::task(&app, &account).await;

        let mut conn = get("/api/accounts/not-an-account/tasks")
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
        let task1 = fixtures::task(&app, &account).await;
        let task2 = fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let tasks: Vec<Task> = conn.response_json().await;
        assert_eq!(vec![task1, task2], tasks);
        Ok(())
    }
}

mod create {
    use divviup_api::{aggregator_api_mock::random_hpke_config, entity::task::HpkeConfig};

    use super::{test, *};

    fn valid_task_json() -> Value {
        json!({
            "name": "my task name",
            "partner": "partner",
            "vdaf": { "type": "count" },
            "min_batch_size": 500,
            "is_leader": true,
            "time_precision_seconds": 60,
            "hpke_config": HpkeConfig::from(random_hpke_config())
        })
    }

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json())
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let task: Task = conn.response_json().await;
        assert_eq!(task.partner, "partner");
        assert_eq!(task.vdaf, json!({"type": "count"}));
        assert_eq!(task.min_batch_size, 500);
        assert!(task.is_leader);
        assert_eq!(task.time_precision_seconds, 60);
        assert!(Tasks::find_by_id(task.id).one(app.db()).await?.is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(json!({
                "name": "my task name",
                "partner": "partner",
                "vdaf": { "type": "poplar1" },
                "min_batch_size": 50,
                "time_precision_seconds": 1,
                "hpke_config": {
                    "id": 1,
                    "kem_id": 1,
                    "kdf_id": 1,
                    "aead_id": 1,
                    "public_key": "key"
                }
            }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let error: Value = conn.response_json().await;
        assert!(error.get("vdaf").is_some());
        assert!(error.get("min_batch_size").is_some());
        assert!(error.get("time_precision_seconds").is_some());
        assert!(error.get("is_leader").is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json())
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let mut conn = post("/api/accounts/does-not-exist/tasks")
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json())
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(admin)
            .with_request_json(valid_task_json())
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let task: Task = conn.response_json().await;
        assert!(Tasks::find_by_id(task.id).one(app.db()).await?.is_some());
        Ok(())
    }
}

mod show {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task, task);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
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
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task, task);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_task(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = get("/api/tasks/some-made-up-id")
            .with_api_headers()
            .with_state(user)
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
        let task = fixtures::task(&app, &account).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task.name, new_name);
        assert_eq!(
            Tasks::find_by_id(task.id)
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
        let task = fixtures::task(&app, &account).await;

        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());

        assert_eq!(
            Tasks::find_by_id(task.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            task.name // unchanged
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;

        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        assert_eq!(
            Tasks::find_by_id(task.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            task.name // unchanged
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task.name, new_name);
        assert_eq!(
            Tasks::find_by_id(task.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            new_name
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_task(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = patch("/api/tasks/not-a-task-id")
            .with_api_headers()
            .with_request_json(json!({ "name": "irrelevant" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }
}
