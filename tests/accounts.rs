mod harness;
use harness::*;

mod get_accounts {
    use super::*;
    #[test]
    fn as_member() {
        set_up(|app| async move {
            let user = test_user();
            let account_with_membership = Account::build("my account".into())?
                .insert(app.db())
                .await?;

            let _other_account = Account::build("someone else's account".into())?
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &account_with_membership)?
                .insert(app.db())
                .await?;

            let mut conn = get("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_ok!(conn);

            let accounts: Vec<Account> = json_response(&mut conn).await;

            assert_eq!(accounts.len(), 1);

            assert_eq!(accounts, vec![account_with_membership]);

            Ok(())
        });
    }

    #[test]
    fn as_admin() {
        set_up(|app| async move {
            let user = test_user();
            let admin_account = build_admin_account("my account").insert(app.db()).await?;

            let other_account = Account::build("someone else's account".into())?
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &admin_account)?
                .insert(app.db())
                .await?;

            let mut conn = get("/api/accounts")
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            let accounts: Vec<Account> = json_response(&mut conn).await;

            assert_eq!(accounts.len(), 2);

            assert_eq!(accounts, vec![admin_account, other_account]);

            Ok(())
        });
    }
}

mod get_account {
    use super::*;
    #[test]
    fn as_a_member() {
        set_up(|app| async move {
            let user = test_user();
            let account_with_membership = Account::build("my account".into())?
                .insert(app.db())
                .await?;

            let _other_account = Account::build("someone else's account".into())?
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &account_with_membership)?
                .insert(app.db())
                .await?;

            let mut conn = get(format!("/api/accounts/{}", account_with_membership.id))
                .with_request_header(KnownHeaderName::Accept, APP_CONTENT_TYPE)
                .with_state(user)
                .run_async(&app)
                .await;

            assert_ok!(conn);
            let account: Account = json_response(&mut conn).await;

            assert_eq!(account, account_with_membership);

            Ok(())
        });
    }

    #[test]
    fn not_as_a_member() {
        set_up(|app| async move {
            let user = test_user();
            let account_with_membership = Account::build("my account".into())?
                .insert(app.db())
                .await?;

            let other_account = Account::build("someone else's account".into())?
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &account_with_membership)?
                .insert(app.db())
                .await?;

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
            let user = test_user();
            let admin_account = build_admin_account("admin account")
                .insert(app.db())
                .await?;

            let other_account = Account::build("someone else's account".into())?
                .insert(app.db())
                .await?;

            let _membership = Membership::build(user.email.clone(), &admin_account)?
                .insert(app.db())
                .await?;

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
