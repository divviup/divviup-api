mod harness;
use divviup_api::{
    entity::queue::Entity,
    queue::{dequeue_one, CreateUser, Job, JobStatus, ResetPassword, SendInvitationEmail},
};
use harness::{test, *};

#[test(harness = with_client_logs)]
async fn create_account(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let account = fixtures::account(&app).await;
    let email = format!("test-{}@example.test", fixtures::random_name());
    let membership = Membership::build(email, &account)?.insert(app.db()).await?;
    let membership_id = membership.id;
    let mut job = CreateUser { membership_id };
    let next = job.perform(&app.config().into(), app.db()).await?.unwrap();
    let create_user_request = client_logs.last();
    assert_eq!(
        create_user_request.url,
        app.config().auth_url.join("/api/v2/users").unwrap()
    );
    let user_id = create_user_request.response_json()["user_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        next,
        ResetPassword {
            membership_id,
            user_id
        }
    );
    Ok(())
}

#[test(harness = with_client_logs)]
async fn reset_password(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let account = fixtures::account(&app).await;
    let email = format!("test-{}@example.test", fixtures::random_name());
    let membership = Membership::build(email, &account)?.insert(app.db()).await?;
    let membership_id = membership.id;
    let mut job = ResetPassword {
        membership_id,
        user_id: fixtures::random_name(),
    };

    let next = job.perform(&app.config().into(), app.db()).await?.unwrap();

    let reset_request = client_logs.last();
    assert_eq!(
        reset_request.url,
        app.config()
            .auth_url
            .join("/api/v2/tickets/password-change")
            .unwrap()
    );
    let action_url = reset_request.response_json()["ticket"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(
        next,
        SendInvitationEmail {
            membership_id,
            action_url
        }
    );
    Ok(())
}

#[test(harness = with_client_logs)]
async fn send_email(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let account = fixtures::account(&app).await;
    let email = format!("test-{}@example.test", fixtures::random_name());
    let membership = Membership::build(email, &account)?.insert(app.db()).await?;
    let membership_id = membership.id;
    let mut job = SendInvitationEmail {
        membership_id,
        action_url: Url::parse("http://any.url/for-now").unwrap(),
    };

    assert!(job.perform(&app.config().into(), app.db()).await?.is_none());

    let reset_request = client_logs.logs().last().unwrap().clone();
    assert_eq!(reset_request.method, Method::Post);
    assert_eq!(
        reset_request.url,
        app.config()
            .postmark_url
            .join("/email/withTemplate")
            .unwrap()
    );
    Ok(())
}

#[test(harness = with_client_logs)]
async fn all_together(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let account = fixtures::account(&app).await;
    let email = format!("test-{}@example.test", fixtures::random_name());
    let membership = Membership::build(email, &account)?.insert(app.db()).await?;

    Job::new_invitation_flow(&membership)
        .insert(app.db())
        .await?;

    let shared_job_state = app.config().into();

    while dequeue_one(app.db(), &Default::default(), &shared_job_state)
        .await?
        .is_some()
    {}

    let full_queue = Entity::find().all(app.db()).await?;
    assert_eq!(full_queue.len(), 3);
    assert!(full_queue.iter().all(|q| q.status == JobStatus::Success));
    let logs = client_logs.logs();
    assert_eq!(
        logs.iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>(),
        &[
            "POST https://auth.example/oauth/token: 200 OK",
            "POST https://auth.example/api/v2/users: 200 OK",
            "POST https://auth.example/api/v2/tickets/password-change: 200 OK",
            "POST https://postmark.example/email/withTemplate: 200 OK"
        ]
    );
    Ok(())
}
