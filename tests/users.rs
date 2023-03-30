mod harness;
use harness::*;

mod get_users_me {
    use super::*;

    #[test]
    fn as_a_logged_in_user() {
        set_up(|app| async move {
            let user = test_user();
            let mut conn = get("/api/users/me")
                .with_request_header(
                    KnownHeaderName::Accept,
                    "application/vnd.divviup+json;version=0.1",
                )
                .with_state(user.clone())
                .run_async(&app)
                .await;

            let response_user: User = json_response(&mut conn).await;

            assert_eq!(user, response_user);
            assert!(!response_user.is_admin());
            Ok(())
        });
    }

    #[test]
    fn as_an_admin() {
        set_up(|app| async move {
            let user = test_user();
            let account = build_admin_account("admin account")
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &account)
                .unwrap()
                .insert(app.db())
                .await?;

            let mut conn = get("/api/users/me")
                .with_request_header(
                    KnownHeaderName::Accept,
                    "application/vnd.divviup+json;version=0.1",
                )
                .with_state(user.clone())
                .run_async(&app)
                .await;

            let response_user: User = json_response(&mut conn).await;

            assert!(response_user.is_admin());
            Ok(())
        });
    }
}
