use crate::harness::{assert_eq, test, *};

#[test(harness = with_configured_client)]
async fn api_tokens_list(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let tokens = client.api_tokens(account.id).await?;
    assert_eq!(tokens.len(), 1);
    Ok(())
}

#[test(harness = with_configured_client)]
async fn create_api_token(
    _app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let token = client.create_api_token(account.id).await?;
    assert!(token.token.is_some());
    Ok(())
}

#[test(harness = with_configured_client)]
async fn delete_api_token(
    app: Arc<DivviupApi>,
    account: Account,
    client: DivviupClient,
) -> TestResult {
    let (token, _) = fixtures::api_token(&app, &account).await;
    client.delete_api_token(token.id).await?;
    assert!(ApiTokens::find_by_id(token.id)
        .one(app.db())
        .await?
        .unwrap()
        .is_tombstoned());
    Ok(())
}
