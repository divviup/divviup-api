use super::*;
pub use divviup_api::api_mocks::aggregator_api::random_hpke_config;
use divviup_api::{
    clients::aggregator_client::api_types::TaskId,
    entity::{
        aggregator::{Feature, Features},
        codec::Codec,
    },
};
use rand::random;
use uuid::Uuid;

pub fn user() -> User {
    User {
        email: random_email(),
        email_verified: true,
        name: "test user".into(),
        nickname: "testy".into(),
        picture: None,
        sub: "".into(),
        updated_at: time::OffsetDateTime::now_utc(),
        admin: None,
    }
}

pub fn random_email() -> String {
    format!("test-{}@example.example", random_name())
}

pub fn random_name() -> String {
    std::iter::repeat_with(fastrand::alphabetic)
        .take(10)
        .collect()
}

pub async fn account(app: &DivviupApi) -> Account {
    Account::build(random_name())
        .unwrap()
        .insert(app.db())
        .await
        .unwrap()
}

pub async fn admin_account(app: &DivviupApi) -> Account {
    let mut active_model = Account::build(random_name()).unwrap();
    active_model.admin = ActiveValue::Set(true);
    active_model.insert(app.db()).await.unwrap()
}

pub async fn membership(app: &DivviupApi, account: &Account, user: &User) -> Membership {
    Membership::build(user.email.clone(), account)
        .unwrap()
        .insert(app.db())
        .await
        .unwrap()
}

pub async fn build_membership(app: &DivviupApi) -> Membership {
    let account = account(app).await;
    let email = format!("test-{}@example.test", random_name());
    Membership::build(email, &account)
        .unwrap()
        .insert(app.db())
        .await
        .unwrap()
}

pub async fn admin(app: &DivviupApi) -> (User, Account, Membership) {
    let user = user();
    let account = admin_account(app).await;
    let membership = membership(app, &account, &user).await;

    (user, account, membership)
}

pub async fn member(app: &DivviupApi) -> (User, Account, Membership) {
    let user = user();
    let account = account(app).await;
    let membership = membership(app, &account, &user).await;

    (user, account, membership)
}

pub async fn collector_credential(app: &DivviupApi, account: &Account) -> CollectorCredential {
    let (token, token_hash) = CollectorCredential::new_token();
    CollectorCredential {
        hpke_config: Codec::new(random_hpke_config()).unwrap(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        id: Uuid::new_v4(),
        account_id: account.id,
        deleted_at: None,
        name: Some(random_name()),
        token: Some(token),
        token_hash: Some(token_hash),
    }
    .into_active_model()
    .insert(app.db())
    .await
    .unwrap()
}

pub async fn task(app: &DivviupApi, account: &Account) -> Task {
    let leader_aggregator = aggregator(app, Some(account)).await;
    let helper_aggregator = aggregator(app, None).await;
    let collector_credential = collector_credential(app, account).await;

    Task {
        id: random::<TaskId>().to_string(),
        account_id: account.id,
        name: random_name(),
        vdaf: task::vdaf::Vdaf::Count.into(),
        min_batch_size: 100,
        max_batch_size: Some(200),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        time_precision_seconds: 60,
        report_count: 0,
        aggregate_collection_count: 0,
        expiration: Some(
            OffsetDateTime::now_utc() + divviup_api::entity::task::DEFAULT_EXPIRATION_DURATION,
        ),
        leader_aggregator_id: leader_aggregator.id,
        helper_aggregator_id: helper_aggregator.id,
        collector_credential_id: collector_credential.id,
        report_counter_interval_collected: 0,
        report_counter_decode_failure: 0,
        report_counter_decrypt_failure: 0,
        report_counter_expired: 0,
        report_counter_outdated_key: 0,
        report_counter_success: 0,
        report_counter_too_early: 0,
        report_counter_task_expired: 0,
    }
    .into_active_model()
    .insert(app.db())
    .await
    .unwrap()
}

pub fn new_aggregator() -> NewAggregator {
    NewAggregator {
        name: Some(format!("{}-aggregator", random_name())),
        api_url: Some(format!("https://api.{}.divviup.org/", random_name())),
        bearer_token: Some(random_name()),
        is_first_party: None,
    }
}

pub async fn aggregator_pair(app: &DivviupApi, account: &Account) -> (Aggregator, Aggregator) {
    (
        aggregator(app, Some(account)).await,
        aggregator(app, None).await,
    )
}

pub async fn aggregator(app: &DivviupApi, account: Option<&Account>) -> Aggregator {
    let api_url: Url = format!("https://api.{}.divviup.org/", random_name())
        .parse()
        .unwrap();
    Aggregator {
        account_id: account.map(|a| a.id),
        api_url: api_url.clone().into(),
        dap_url: format!("https://dap.{}.divviup.org/", random_name())
            .parse()
            .unwrap(),
        encrypted_bearer_token: app
            .crypter()
            .encrypt(api_url.as_ref().as_bytes(), random_name().as_bytes())
            .unwrap(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
        is_first_party: account.is_none(),
        id: Uuid::new_v4(),
        name: random_name(),
        role: Role::Either,
        query_types: Default::default(),
        vdafs: Default::default(),
        protocol: Protocol::Dap07,
        features: Features::from(Feature::TokenHash).into(),
    }
    .into_active_model()
    .insert(app.db())
    .await
    .unwrap()
}

pub async fn api_token(app: &DivviupApi, account: &Account) -> (ApiToken, HeaderValue) {
    let (api_token, token) = ApiToken::build(account);
    let api_token = api_token.insert(app.db()).await.unwrap();
    (api_token, format!("Bearer {token}").into())
}

pub async fn admin_token(app: &DivviupApi) -> HeaderValue {
    let account = admin_account(app).await;
    let (_, header) = api_token(app, &account).await;
    header
}

pub async fn make_account_admin(app: &DivviupApi, account: Account) -> Account {
    let mut account = account.into_active_model();
    account.admin = ActiveValue::Set(true);
    account.update(app.db()).await.unwrap()
}
