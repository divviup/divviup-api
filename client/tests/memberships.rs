mod harness;
use harness::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn membership_list(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let memberships = vec![
        fixtures::membership(&app, &account, &fixtures::user()).await,
        fixtures::membership(&app, &account, &fixtures::user()).await,
        fixtures::membership(&app, &account, &fixtures::user()).await,
    ];

    let response_memberships = client.memberships(account.id).await?;
    assert_eq!(
        memberships
            .iter()
            .map(|m| (m.id, m.user_email.as_ref()))
            .collect::<Vec<_>>(),
        response_memberships
            .iter()
            .map(|m| (m.id, m.user_email.as_ref()))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_membership(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let email = fixtures::random_email();
    let response_membership = client.create_membership(account.id, &email).await?;
    assert_eq!(response_membership.user_email.as_ref(), &*email);
    assert_eq!(response_membership.account_id, account.id);
    assert_eq!(
        Memberships::find_by_id(response_membership.id)
            .one(app.db())
            .await?
            .unwrap()
            .user_email,
        email
    );

    Ok(())
}

#[test(harness = with_configured_client)]
async fn delete_membership(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let membership = fixtures::membership(&app, &account, &fixtures::user()).await;
    client.delete_membership(membership.id).await?;
    assert!(membership.reload(app.db()).await?.is_none());
    Ok(())
}
