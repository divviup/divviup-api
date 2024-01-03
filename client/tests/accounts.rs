mod harness;
use harness::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn account_list(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let accounts = client.accounts().await?;
    assert_eq!(
        accounts.into_iter().map(|a| a.id).collect::<Vec<_>>(),
        vec![account.id]
    );
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_account(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    fixtures::make_account_admin(&app, account).await;
    let name = fixtures::random_name();
    let account = client.create_account(&name).await?;
    assert_eq!(
        Accounts::find_by_id(account.id)
            .one(app.db())
            .await?
            .unwrap()
            .name,
        name
    );
    Ok(())
}

#[test(harness = with_configured_client)]
async fn rename_account(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let name = fixtures::random_name();
    let account = client.rename_account(account.id, &name).await?;
    assert_eq!(&account.name, &name);
    assert_eq!(
        Accounts::find_by_id(account.id)
            .one(app.db())
            .await?
            .unwrap()
            .name,
        name
    );
    Ok(())
}
