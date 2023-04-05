mod harness;
use harness::*;

mod create {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let mut conn = post(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
            .with_request_json(json!({ "user_email": "someone.else@example.com" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);

        let membership: Membership = conn.response_json().await;
        assert_eq!(membership.user_email, "someone.else@example.com");
        assert_eq!(membership.account_id, account.id);

        assert!(Memberships::find_by_id(membership.id)
            .one(app.db())
            .await?
            .is_some());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn invalid(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let membership_count_before = Memberships::find().count(app.db()).await?;

        let mut conn = post(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
            .with_request_json(json!({ "user_email": "not a valid email" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 400);

        let errors: Value = conn.response_json().await;
        assert!(errors.get("user_email").is_some());

        assert_eq!(
            membership_count_before,
            Memberships::find().count(app.db()).await?
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let account = fixtures::account(&app).await;
        let conn = post(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
            .with_request_json(json!({ "user_email": "someone.else@example.com" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_eq!(conn.status().unwrap_or(Status::NotFound), 404);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let conn = post("/api/accounts/no-account-with-this-id/memberships")
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
            .with_request_json(json!({ "user_email": "someone.else@example.com" }))
            .with_state(user)
            .run_async(&app)
            .await;
        assert_eq!(conn.status().unwrap_or(Status::NotFound), 404);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
            .with_request_json(json!({ "user_email": "someone.else@example.com" }))
            .with_state(user)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);

        let membership: Membership = conn.response_json().await;
        assert_eq!(membership.user_email, "someone.else@example.com");
        assert_eq!(membership.account_id, account.id);

        assert!(Memberships::find_by_id(membership.id)
            .one(app.db())
            .await?
            .is_some());

        Ok(())
    }
}

mod index {
    use super::{test, *};

    #[test(harness = set_up)]

    async fn member(app: DivviupApi) -> TestResult {
        let _ = fixtures::member(&app).await; // there is unrelated data in the db;

        let (user, account, ..) = fixtures::member(&app).await;
        fixtures::membership(&app, &account, &fixtures::user()).await;
        fixtures::membership(&app, &account, &fixtures::user()).await;
        let mut conn = get(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let memberships: Vec<Membership> = conn.response_json().await;
        assert_eq!(memberships.len(), 3);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let (_, account, ..) = fixtures::member(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = get(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = get("/api/accounts/not-an-id/memberships")
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await; // there is unrelated data in the db;
        let account = fixtures::account(&app).await;
        fixtures::membership(&app, &account, &fixtures::user()).await;
        fixtures::membership(&app, &account, &fixtures::user()).await;

        let mut conn = get(format!("/api/accounts/{}/memberships", account.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        let memberships: Vec<Membership> = conn.response_json().await;
        assert_eq!(memberships.len(), 2);

        Ok(())
    }
}

mod delete {
    use super::{test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let other_membership = fixtures::membership(&app, &account, &fixtures::user()).await;
        fixtures::membership(&app, &account, &fixtures::user()).await;
        let conn = delete(format!("/api/memberships/{}", other_membership.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 204);
        assert!(Memberships::find_by_id(other_membership.id)
            .one(app.db())
            .await?
            .is_none());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let account = fixtures::account(&app).await;
        let other_membership = fixtures::membership(&app, &account, &fixtures::user()).await;
        let mut conn = delete(format!("/api/memberships/{}", other_membership.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(Memberships::find_by_id(other_membership.id)
            .one(app.db())
            .await?
            .is_some());
        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_id(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete("/api/memberships/876b2071-9da8-4bda-bd4c-8d42a3ae7d90")
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn removing_self(app: DivviupApi) -> TestResult {
        let (user, _, membership) = fixtures::member(&app).await;
        let conn = delete(format!("/api/memberships/{}", membership.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 403);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let other_membership = fixtures::membership(&app, &account, &fixtures::user()).await;
        let conn = delete(format!("/api/memberships/{}", other_membership.id))
            .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 204);
        assert!(Memberships::find_by_id(other_membership.id)
            .one(app.db())
            .await?
            .is_none());
        Ok(())
    }
}
