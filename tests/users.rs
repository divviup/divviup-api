mod harness;
use harness::*;

mod get_users_me {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_a_logged_in_user(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let mut conn = get("/api/users/me")
            .with_api_headers()
            .with_state(user.clone())
            .run_async(&app)
            .await;

        let mut response_user: User = conn.response_json().await;
        assert!(!response_user.is_admin());

        response_user.admin = None; // for equality comparison
        assert_eq!(user, response_user);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_an_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let mut conn = get("/api/users/me")
            .with_api_headers()
            .with_state(admin.clone())
            .run_async(&app)
            .await;

        let mut response_user: User = conn.response_json().await;
        assert!(response_user.is_admin());

        response_user.admin = None; // for equality comparison
        assert_eq!(admin, response_user);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_integration_testing_user(app: DivviupApi) -> TestResult {
        let user = User::for_integration_testing();
        let mut conn = get("/api/users/me")
            .with_api_headers()
            .with_state(user.clone())
            .run_async(&app)
            .await;

        let response_user: User = conn.response_json().await;
        assert!(response_user.is_admin());

        assert_eq!(user, response_user);
        Ok(())
    }
}
