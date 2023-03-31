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
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let tasks: Vec<Task> = json_response(&mut conn).await;
        assert_eq!(vec![task1, task2], tasks);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }
}

mod create {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn success(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }
}

mod show {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_task(_app: DivviupApi) -> TestResult {
        Ok(())
    }
}

mod update {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn valid(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(_app: DivviupApi) -> TestResult {
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_task(_app: DivviupApi) -> TestResult {
        Ok(())
    }
}
