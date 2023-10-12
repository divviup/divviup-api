use divviup_api::clients::aggregator_client::*;
use test_support::*;

mod index {
    use super::{assert_eq, test, *};

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

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let task1 = fixtures::task(&app, &account).await;
        let task2 = fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let tasks: Vec<Task> = conn.response_json().await;
        assert_eq!(vec![task1, task2], tasks);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let task1 = fixtures::task(&app, &account).await;
        let task2 = fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let tasks: Vec<Task> = conn.response_json().await;
        assert_eq!(vec![task1, task2], tasks);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        fixtures::task(&app, &account).await;
        fixtures::task(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/tasks", account.id))
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
    use divviup_api::entity::task::vdaf::Vdaf;

    fn valid_task_json(
        collector_credential: &CollectorCredential,
        leader_aggregator: &Aggregator,
        helper_aggregator: &Aggregator,
    ) -> Value {
        json!({
            "name": "my task name",
            "leader_aggregator_id": leader_aggregator.id,
            "helper_aggregator_id": helper_aggregator.id,
            "vdaf": { "type": "count" },
            "min_batch_size": 500,
            "time_precision_seconds": 60,
            "collector_credential_id": collector_credential.id
        })
    }

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let task: Task = conn.response_json().await;

        assert_eq!(task.leader_aggregator_id, leader.id);
        assert_eq!(task.helper_aggregator_id, helper.id);
        assert_eq!(task.vdaf, Vdaf::Count);
        assert_eq!(task.min_batch_size, 500);
        assert_eq!(task.time_precision_seconds, 60);
        assert!(task.reload(app.db()).await?.is_some());

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
                "vdaf": { "type": "poplar1" },
                "min_batch_size": 50,
                "time_precision_seconds": 1,
                "collector_credential": ""
            }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let error: Value = conn.response_json().await;
        assert!(error.get("vdaf").is_some());
        assert!(error.get("min_batch_size").is_some());
        assert!(error.get("time_precision_seconds").is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let mut conn = post("/api/accounts/does-not-exist/tasks")
            .with_api_headers()
            .with_state(user)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_state(admin)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let task: Task = conn.response_json().await;
        assert!(task.reload(app.db()).await?.is_some());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let task: Task = conn.response_json().await;
        assert!(task.reload(app.db()).await?.is_some());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;
        let count_before = Tasks::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;
        let count_after = Tasks::find().count(app.db()).await?;
        assert_response!(conn, 201);
        assert_eq!(count_before + 1, count_after);
        let task: Task = conn.response_json().await;
        assert!(task.reload(app.db()).await?.is_some());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        let (leader, helper) = fixtures::aggregator_pair(&app, &account).await;
        let collector_credential = fixtures::collector_credential(&app, &account).await;

        let count_before = Tasks::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/tasks", account.id))
            .with_api_headers()
            .with_auth_header(token)
            .with_request_json(valid_task_json(&collector_credential, &leader, &helper))
            .run_async(&app)
            .await;

        let count_after = Tasks::find().count(app.db()).await?;
        assert_eq!(count_before, count_after);
        assert_not_found!(conn);
        Ok(())
    }
}

mod show {
    use super::{assert_eq, test, *};
    use time::Duration;

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

    #[test(harness = with_client_logs)]
    async fn metrics_caching(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut task = task.into_active_model();
        task.updated_at = ActiveValue::Set(OffsetDateTime::now_utc() - Duration::minutes(10));

        let task = task.update(app.db()).await?;

        let first_party_aggregator = task.first_party_aggregator(app.db()).await?.unwrap();

        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_state(user.clone())
            .run_async(&app)
            .await;
        assert_ok!(conn);

        let aggregator_api_request = client_logs.last();
        assert_eq!(
            aggregator_api_request.url,
            first_party_aggregator
                .api_url
                .join(&format!("tasks/{}/metrics", task.id))
                .unwrap()
        );
        let metrics: TaskMetrics = aggregator_api_request.response_json();

        let response_task: Task = conn.response_json().await;

        assert_eq!(response_task.report_count, metrics.reports as i32);
        assert_eq!(
            response_task.aggregate_collection_count,
            metrics.report_aggregations as i32
        );
        assert!(response_task.updated_at > task.updated_at);

        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        let second_response_task: Task = conn.response_json().await;
        assert_eq!(
            second_response_task.report_count,
            response_task.report_count
        );
        assert_eq!(
            second_response_task.aggregate_collection_count,
            response_task.aggregate_collection_count
        );
        assert_eq!(second_response_task.updated_at, response_task.updated_at);

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

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task, task);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task, task);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }
}

mod update {
    use super::{assert_eq, test, *};

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
        assert_eq!(task.reload(app.db()).await?.unwrap().name, new_name);

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
            task.reload(app.db()).await?.unwrap().name,
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
            task.reload(app.db()).await?.unwrap().name,
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
        assert_eq!(task.reload(app.db()).await?.unwrap().name, new_name);
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

    #[test(harness = set_up)]
    async fn admin_token(app: DivviupApi) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task.name, new_name);
        assert_eq!(task.reload(app.db()).await?.unwrap().name, new_name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn member_token(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let task = fixtures::task(&app, &account).await;

        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let response_task: Task = conn.response_json().await;
        assert_eq!(response_task.name, new_name);
        assert_eq!(task.reload(app.db()).await?.unwrap().name, new_name);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonmember_token(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let name_before = task.name.clone();
        let new_name = format!("new name {}", fixtures::random_name());
        let mut conn = patch(format!("/api/tasks/{}", task.id))
            .with_api_headers()
            .with_request_json(json!({ "name": &new_name }))
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert_eq!(task.reload(app.db()).await?.unwrap().name, name_before);
        assert_not_found!(conn);
        Ok(())
    }
}

mod collector_auth_tokens {
    use divviup_api::clients::aggregator_client::api_types::AuthenticationToken;

    use super::{assert_eq, test, *};

    #[test(harness = with_client_logs)]
    async fn as_member(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        let auth_token = client_logs
            .last()
            .response_json::<TaskResponse>()
            .collector_auth_token
            .unwrap();

        assert_ok!(conn);
        let body: Vec<AuthenticationToken> = conn.response_json().await;
        assert_eq!(vec![auth_token], body);
        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn as_rando(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert!(client_logs.logs().is_empty());
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn as_admin(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        let auth_token = client_logs
            .last()
            .response_json::<TaskResponse>()
            .collector_auth_token
            .unwrap();

        assert_ok!(conn);
        let body: Vec<AuthenticationToken> = conn.response_json().await;
        assert_eq!(vec![auth_token], body);
        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn admin_token(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let token = fixtures::admin_token(&app).await;
        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        let auth_token = client_logs
            .last()
            .response_json::<TaskResponse>()
            .collector_auth_token
            .unwrap();

        assert_ok!(conn);
        let body: Vec<AuthenticationToken> = conn.response_json().await;
        assert_eq!(vec![auth_token], body);
        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn member_token(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &account).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;
        let auth_token = client_logs
            .last()
            .response_json::<TaskResponse>()
            .collector_auth_token
            .unwrap();

        assert_ok!(conn);
        let body: Vec<AuthenticationToken> = conn.response_json().await;
        assert_eq!(vec![auth_token], body);
        Ok(())
    }

    #[test(harness = with_client_logs)]
    async fn nonmember_token(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let (_, token) = fixtures::api_token(&app, &other_account).await;

        let account = fixtures::account(&app).await;
        let task = fixtures::task(&app, &account).await;
        let mut conn = get(format!("/api/tasks/{}/collector_auth_tokens", task.id))
            .with_api_headers()
            .with_auth_header(token)
            .run_async(&app)
            .await;

        assert!(client_logs.logs().is_empty());
        assert_not_found!(conn);
        Ok(())
    }
}
