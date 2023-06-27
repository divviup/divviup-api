mod harness;
use divviup_api::{
    entity::queue::Entity,
    queue::{CreateUser, Job, JobStatus, Queue, ResetPassword, SendInvitationEmail, V1},
};
use harness::{assert_eq, test, *};
use uuid::Uuid;

#[test(harness = with_client_logs)]
async fn create_account(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let membership = fixtures::build_membership(&app).await;
    let membership_id = membership.id;
    let mut job = CreateUser { membership_id };
    let next = job.perform(&app.config().into(), app.db()).await?.unwrap();
    let create_user_request = client_logs.last();
    assert_eq!(
        create_user_request.url,
        app.config().auth_url.join("/api/v2/users").unwrap()
    );
    let user_id = create_user_request.response_json::<Value>()["user_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        next.job,
        ResetPassword {
            membership_id,
            user_id
        }
    );
    Ok(())
}

#[test(harness = with_client_logs)]
async fn reset_password(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let membership = fixtures::build_membership(&app).await;
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
    let action_url = reset_request.response_json::<Value>()["ticket"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let Job::V1(V1::SendInvitationEmail(next)) = next.job else { panic!() };
    assert_eq!(next.membership_id, membership_id);
    assert_eq!(next.action_url, action_url);
    Ok(())
}

#[test(harness = with_client_logs)]
async fn send_email(app: DivviupApi, client_logs: ClientLogs) -> TestResult {
    let membership = fixtures::build_membership(&app).await;
    let membership_id = membership.id;
    let mut job = SendInvitationEmail {
        membership_id,
        action_url: Url::parse("http://any.url/for-now").unwrap(),
        message_id: Uuid::new_v4(),
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
    let membership = fixtures::build_membership(&app).await;

    Job::new_invitation_flow(&membership)
        .insert(app.db())
        .await?;

    let mut completed_queue_jobs = vec![];
    let queue = Queue::new(app.db(), app.config());
    while let Some(queue_job) = queue.perform_one_queue_job().await? {
        completed_queue_jobs.push(queue_job);
    }

    let full_queue = Entity::find().all(app.db()).await?;
    assert_eq!(completed_queue_jobs, full_queue);
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

#[test]
fn json_representations() {
    let membership_id = Uuid::new_v4();
    assert_eq!(
        serde_json::to_value(Job::from(CreateUser { membership_id })).unwrap(),
        json!({
            "version": "V1",
            "type": "CreateUser",
            "membership_id": membership_id
        })
    );

    let message_id = Uuid::new_v4();
    let action_url: Url = "https://action.url".parse().unwrap();

    assert_eq!(
        serde_json::to_value(Job::from(SendInvitationEmail {
            membership_id,
            action_url: action_url.clone(),
            message_id
        }))
        .unwrap(),
        json!({
            "version": "V1",
            "type": "SendInvitationEmail",
            "membership_id": membership_id,
            "action_url": action_url,
            "message_id": message_id
        })
    );

    let user_id: String = "user-id".into();
    assert_eq!(
        serde_json::to_value(Job::from(ResetPassword {
            membership_id,
            user_id: user_id.clone()
        }))
        .unwrap(),
        json!({
            "version": "V1",
            "type": "ResetPassword",
            "membership_id": membership_id,
            "user_id": user_id
        })
    );
}
