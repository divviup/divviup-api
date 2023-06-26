mod harness;
use harness::*;

#[trillium::async_trait]
impl Reload for ApiToken {
    async fn reload(self, db: &impl ConnectionTrait) -> Result<Option<Self>, DbErr> {
        ApiTokens::find_by_id(self.id).one(db).await
    }
}

mod index {
    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let other_account = fixtures::account(&app).await;
        let _ = fixtures::api_token(&app, &other_account).await;
        let (user, account, ..) = fixtures::member(&app).await;
        let (token1, _) = fixtures::api_token(&app, &account).await;
        let (token2, _) = fixtures::api_token(&app, &account).await;
        let (deleted, _) = fixtures::api_token(&app, &account).await;
        let deleted = deleted.tombstone().update(app.db()).await.unwrap();

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_ok!(conn);

        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(&api_tokens, &vec![token1, token2, deleted]);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::api_token(&app, &account).await;
        fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();

        let account = fixtures::account(&app).await;
        fixtures::api_token(&app, &account).await;
        fixtures::api_token(&app, &account).await;

        let mut conn = get("/api/accounts/not-an-account/api_tokens")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token1, _) = fixtures::api_token(&app, &account).await;
        let (api_token2, _) = fixtures::api_token(&app, &account).await;

        let mut conn = get(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_ok!(conn);
        let api_tokens: Vec<ApiToken> = conn.response_json().await;
        assert_same_json_representation(&api_tokens, &vec![api_token1, api_token2]);
        Ok(())
    }
}

mod create {
    use super::{assert_eq, test, *};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};

    #[test(harness = set_up)]
    async fn success(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;

        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_response!(conn, 201);
        let mut response: ApiToken = conn.response_json().await;
        let api_token = ApiTokens::find_by_id(response.id)
            .one(app.db())
            .await?
            .unwrap();

        assert_eq!(&api_token.token_hash, &response.token_hash);

        assert_eq!(
            &*Sha256::digest(
                URL_SAFE_NO_PAD
                    .decode(response.token.take().unwrap())
                    .unwrap()
            ),
            &*api_token.token_hash
        );

        assert_same_json_representation(&response, &api_token);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn not_member(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let account = fixtures::account(&app).await; // no membership

        let api_token_count_before = ApiTokens::find().count(app.db()).await?;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let api_token_count_after = ApiTokens::find().count(app.db()).await?;
        assert_eq!(api_token_count_before, api_token_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn nonexistant_account(app: DivviupApi) -> TestResult {
        let user = fixtures::user();
        let api_token_count_before = ApiTokens::find().count(app.db()).await?;

        let mut conn = post("/api/accounts/does-not-exist/api_tokens")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_not_found!(conn);
        let api_token_count_after = ApiTokens::find().count(app.db()).await?;
        assert_eq!(api_token_count_before, api_token_count_after);

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let mut conn = post(format!("/api/accounts/{}/api_tokens", account.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;

        assert_response!(conn, 201);
        let mut api_token: ApiToken = conn.response_json().await;
        let api_token_from_db = ApiTokens::find_by_id(api_token.id)
            .one(app.db())
            .await?
            .unwrap();

        assert_eq!(
            &*Sha256::digest(
                URL_SAFE_NO_PAD
                    .decode(api_token.token.take().unwrap())
                    .unwrap()
            ),
            &*api_token_from_db.token_hash
        );

        assert_same_json_representation(&api_token, &api_token_from_db);

        Ok(())
    }
}

mod delete {
    use uuid::Uuid;

    use super::{assert_eq, test, *};

    #[test(harness = set_up)]
    async fn nonexistant_api_token(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;
        let mut conn = delete(format!("/api/api_tokens/{}", Uuid::new_v4()))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_member(app: DivviupApi) -> TestResult {
        let (user, account, ..) = fixtures::member(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn non_member(app: DivviupApi) -> TestResult {
        let account = fixtures::account(&app).await;
        let (user, ..) = fixtures::member(&app).await;
        let (api_token, ..) = fixtures::api_token(&app, &account).await;
        let mut conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;
        assert_not_found!(conn);
        assert!(!api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }

    #[test(harness = set_up)]
    async fn admin_not_member(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let account = fixtures::account(&app).await;
        let (api_token, _) = fixtures::api_token(&app, &account).await;
        let conn = delete(format!("/api/api_tokens/{}", api_token.id))
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_status!(conn, 204);
        assert!(api_token.reload(app.db()).await?.unwrap().is_tombstoned());

        Ok(())
    }
}
