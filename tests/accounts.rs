mod harness;
use harness::*;

mod index {
    use super::{test, *};
    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let _other_account = fixtures::account(&app).await;

        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let accounts: Vec<Account> = conn.response_json().await;
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts, vec![account]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_admin(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = get("/api/accounts")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        let accounts: Vec<Account> = conn.response_json().await;

        assert_eq!(accounts.len(), 2);

        assert_eq!(accounts, vec![account, other_account]);

        Ok(())
    }
}

mod show {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_a_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let mut conn = get(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let account_response: Account = conn.response_json().await;
        assert_eq!(account_response, account);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member_but_as_an_admin(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;

        let mut conn = get(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let account: Account = conn.response_json().await;

        assert_eq!(account, other_account);

        Ok(())
    }
}

mod create {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn not_logged_in(app: DivviupApi) -> TestResult {
        let conn = post("/api/accounts")
            .with_api_headers()
            .with_request_json(json!({ "name": "some account name" }))
            .run_async(&app)
            .await;

        assert_response!(conn, 403);
        let accounts = Accounts::find().all(app.db()).await?;
        assert_eq!(accounts.len(), 0);
        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 0);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn valid(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = post("/api/accounts")
            .with_api_headers()
            .with_state(user.clone())
            .with_request_json(json!({ "name": "some account name" }))
            .run_async(&app)
            .await;
        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;
        assert_eq!(account.name, "some account name");

        let accounts = Accounts::find().all(app.db()).await?;

        assert_eq!(accounts, [account.clone()]);

        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 1);
        assert_eq!(&memberships[0].user_email, &user.email);
        assert_eq!(&memberships[0].account_id, &account.id);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = post("/api/accounts")
            .with_api_headers()
            .with_state(user.clone())
            .with_request_json(json!({ "name": "" }))
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());
        let accounts = Accounts::find().all(app.db()).await?;
        assert_eq!(accounts.len(), 0);
        let memberships = Memberships::find().all(app.db()).await?;
        assert_eq!(memberships.len(), 0);
        Ok(())
    }
}

mod update {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_a_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = patch(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;
        assert_eq!(&account.name, "new name");
        assert_eq!(
            Accounts::find_by_id(account.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            "new name"
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let other_account = fixtures::account(&app).await;
        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_as_a_member_but_as_an_admin(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let other_account = fixtures::account(&app).await;

        let mut conn = patch(format!("/api/accounts/{}", other_account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "new name" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 202);
        let account: Account = conn.response_json().await;

        assert_eq!(&account.name, "new name");
        assert_eq!(
            Accounts::find_by_id(account.id)
                .one(app.db())
                .await?
                .unwrap()
                .name,
            "new name"
        );

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let mut conn = patch(format!("/api/accounts/{}", account.id))
            .with_api_headers()
            .with_request_json(json!({ "name": "" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 400);
        let errors: Value = conn.response_json().await;
        assert!(errors.get("name").is_some());

        Ok(())
    }
}
