mod harness;
use harness::*;

mod index {
    use super::*;
    #[test]
    fn as_member() {
        set_up(|app| async move {
            let (user, account, ..) = fixtures::member(&app).await;
            let _other_account = fixtures::account(&app).await;

            let mut conn = get("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_ok!(conn);
            let accounts: Vec<Account> = json_response(&mut conn).await;
            assert_eq!(accounts.len(), 1);
            assert_eq!(accounts, vec![account]);
            Ok(())
        });
    }

    #[test]
    fn as_admin() {
        set_up(|app| async move {
            let (user, account, ..) = fixtures::admin(&app).await;
            let other_account = fixtures::account(&app).await;
            let mut conn = get("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            let accounts: Vec<Account> = json_response(&mut conn).await;

            assert_eq!(accounts.len(), 2);

            assert_eq!(accounts, vec![account, other_account]);

            Ok(())
        });
    }
}

mod show {
    use super::*;
    #[test]
    fn as_a_member() {
        set_up(|app| async move {
            let (user, account, ..) = fixtures::member(&app).await;
            let mut conn = get(format!("/api/accounts/{}", account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_ok!(conn);
            let account_response: Account = json_response(&mut conn).await;
            assert_eq!(account_response, account);

            Ok(())
        });
    }

    #[test]
    fn not_as_a_member() {
        set_up(|app| async move {
            let (user, ..) = fixtures::member(&app).await;
            let other_account = fixtures::account(&app).await;
            let mut conn = get(format!("/api/accounts/{}", other_account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_eq!(conn.status().unwrap_or(Status::NotFound), Status::NotFound);
            assert!(conn.take_response_body().is_none());

            Ok(())
        });
    }

    #[test]
    fn not_as_a_member_but_as_an_admin() {
        set_up(|app| async move {
            let (user, ..) = fixtures::admin(&app).await;
            let other_account = fixtures::account(&app).await;

            let mut conn = get(format!("/api/accounts/{}", other_account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_ok!(conn);
            let account: Account = json_response(&mut conn).await;

            assert_eq!(account, other_account);

            Ok(())
        });
    }
}

mod create {
    use super::*;

    #[test]
    fn not_logged_in() {
        set_up(|app| async move {
            let conn = post("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_request_body(
                    serde_json::to_string(&json!({
                       "name": "some account name"
                    }))
                    .unwrap(),
                )
                .run_async(&app)
                .await;

            assert_response!(conn, 403);
            let accounts = Accounts::find().all(app.db()).await?;
            assert_eq!(accounts.len(), 0);
            let memberships = Memberships::find().all(app.db()).await?;
            assert_eq!(memberships.len(), 0);

            Ok(())
        })
    }

    #[test]
    fn valid() {
        set_up(|app| async move {
            let user = fixtures::user();
            let mut conn = post("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_state(user.clone())
                .with_request_body(
                    serde_json::to_string(&json!({
                       "name": "some account name"
                    }))
                    .unwrap(),
                )
                .run_async(&app)
                .await;
            assert_response!(conn, 202);
            let account: Account = json_response(&mut conn).await;
            assert_eq!(account.name, "some account name");

            let accounts = Accounts::find().all(app.db()).await?;

            assert_eq!(accounts, [account.clone()]);

            let memberships = Memberships::find().all(app.db()).await?;
            assert_eq!(memberships.len(), 1);
            assert_eq!(&memberships[0].user_email, &user.email);
            assert_eq!(&memberships[0].account_id, &account.id);

            Ok(())
        })
    }

    #[test]
    fn invalid() {
        set_up(|app| async move {
            let user = fixtures::user();
            let mut conn = post("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_state(user.clone())
                .with_request_body(serde_json::to_string(&json!({ "name": "" })).unwrap())
                .run_async(&app)
                .await;

            assert_response!(conn, 400);
            let errors: serde_json::Value = json_response(&mut conn).await;
            assert!(errors.get("name").is_some());
            let accounts = Accounts::find().all(app.db()).await?;
            assert_eq!(accounts.len(), 0);
            let memberships = Memberships::find().all(app.db()).await?;
            assert_eq!(memberships.len(), 0);
            Ok(())
        })
    }
}

mod update {
    use super::*;

    #[test]
    fn as_a_member() {
        set_up(|app| async move {
            let (user, account, ..) = fixtures::member(&app).await;

            let mut conn = patch(format!("/api/accounts/{}", account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_request_body(serde_json::to_string(&json!({ "name": "new name" }))?)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_response!(conn, 202);
            let account: Account = json_response(&mut conn).await;
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
        });
    }

    #[test]
    fn not_as_a_member() {
        set_up(|app| async move {
            let (user, ..) = fixtures::member(&app).await;
            let other_account = fixtures::account(&app).await;
            let mut conn = patch(format!("/api/accounts/{}", other_account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_request_body(serde_json::to_string(&json!({ "name": "new name" }))?)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_eq!(conn.status().unwrap_or(Status::NotFound), Status::NotFound);
            assert!(conn.take_response_body().is_none());

            Ok(())
        });
    }

    #[test]
    fn not_as_a_member_but_as_an_admin() {
        set_up(|app| async move {
            let (user, ..) = fixtures::admin(&app).await;
            let other_account = fixtures::account(&app).await;

            let mut conn = patch(format!("/api/accounts/{}", other_account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_request_body(serde_json::to_string(&json!({ "name": "new name" }))?)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_response!(conn, 202);
            let account: Account = json_response(&mut conn).await;

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
        });
    }

    #[test]
    fn invalid() {
        set_up(|app| async move {
            let (user, account, ..) = fixtures::member(&app).await;
            let mut conn = patch(format!("/api/accounts/{}", account.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_request_header(KnownHeaderName::ContentType, APP_CONTENT_TYPE)
                .with_request_body(serde_json::to_string(&json!({ "name": "" }))?)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_response!(conn, 400);
            let errors: serde_json::Value = json_response(&mut conn).await;
            assert!(errors.get("name").is_some());

            Ok(())
        });
    }
}
