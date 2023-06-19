use super::*;
use divviup_api::{
    aggregator_api_mock::{self, random_hpke_config},
    clients::aggregator_client::TaskCreate,
    entity::aggregator::Role,
};

pub fn user() -> User {
    User {
        email: format!("test-{}@example.example", random_name()),
        email_verified: true,
        name: "test user".into(),
        nickname: "testy".into(),
        picture: None,
        sub: "".into(),
        updated_at: time::OffsetDateTime::now_utc(),
        admin: Some(false),
    }
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
    let account = account(&app).await;
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

pub async fn task(app: &DivviupApi, account: &Account) -> Task {
    let new_task = NewTask {
        name: Some(random_name()),
        partner_url: Some("https://dap.clodflair.test".into()),
        vdaf: Some(task::vdaf::Vdaf::Count),
        min_batch_size: Some(500),
        max_batch_size: Some(10000),
        is_leader: Some(true),
        expiration: None,
        time_precision_seconds: Some(60 * 60),
        hpke_config: Some(encode_hpke_config(random_hpke_config())),
        id: None,
        vdaf_verify_key: None,
        aggregator_auth_token: None,
        collector_auth_token: None,
    };
    new_task.validate().unwrap();
    let task_create = TaskCreate::build(new_task.clone(), app.config()).unwrap();
    let api_response = aggregator_api_mock::task_response(task_create);
    task::build_task(new_task, api_response, account)
        .insert(app.db())
        .await
        .unwrap()
}

pub fn new_aggregator() -> NewAggregator {
    NewAggregator {
        role: Some(Role::Either.as_ref().to_string()),
        name: Some(format!("{}-aggregator", random_name())),
        api_url: Some(format!("https://api.{}.divviup.test", random_name())),
        dap_url: Some(format!("https://dap.{}.divviup.test", random_name())),
        bearer_token: Some(random_name()),
    }
}

pub async fn aggregator(app: &DivviupApi, account: Option<&Account>) -> Aggregator {
    new_aggregator()
        .build(account)
        .unwrap()
        .insert(app.db())
        .await
        .unwrap()
}
