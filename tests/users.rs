mod harness;
use divviup_api::{entity::Membership, User};
use harness::*;
use sea_orm::ActiveModelTrait;
use trillium::KnownHeaderName;

#[test]
fn get_users_me() {
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

        let response_user: User =
            serde_json::from_str(&conn.take_response_body_string().unwrap()).unwrap();

        assert_eq!(user, response_user);
        assert!(!response_user.is_admin());
        Ok(())
    });
}

#[test]
fn get_users_me_when_the_user_is_an_admin() {
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

        let response_user: User =
            serde_json::from_str(&conn.take_response_body_string().unwrap()).unwrap();

        assert!(response_user.is_admin());
        Ok(())
    });
}
