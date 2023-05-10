mod harness;
use harness::{test, *};

mod index {
    use super::{test, *};
    use divviup_api::entity::queue::{JobStatus, Model as QueueItem};

    #[test(harness = set_up)]
    async fn as_an_admin(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        let queue_item = Job::new_invitation_flow(&fixtures::build_membership(&app).await)
            .insert(app.db())
            .await?;
        let mut conn = get("/api/admin/queue")
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await;
        assert_ok!(conn);
        assert_eq!(
            conn.response_json::<Vec<QueueItem>>().await,
            vec![queue_item]
        );
        Ok(())
    }

    #[test(harness = set_up)]
    async fn as_a_non_admin(app: DivviupApi) -> TestResult {
        let (user, ..) = fixtures::member(&app).await;

        Job::new_invitation_flow(&fixtures::build_membership(&app).await)
            .insert(app.db())
            .await?;

        let conn = get("/api/admin/queue")
            .with_api_headers()
            .with_state(user)
            .run_async(&app)
            .await;

        assert_status!(conn, 404);
        Ok(())
    }

    #[test(harness = set_up)]
    async fn filtering(app: DivviupApi) -> TestResult {
        let (admin, ..) = fixtures::admin(&app).await;
        Job::new_invitation_flow(&fixtures::build_membership(&app).await)
            .insert(app.db())
            .await?;

        Queue::from(&app).perform_one_queue_job().await?.unwrap();

        let success: Vec<QueueItem> = get("/api/admin/queue?status=success")
            .with_api_headers()
            .with_state(admin.clone())
            .run_async(&app)
            .await
            .response_json()
            .await;

        let pending: Vec<QueueItem> = get("/api/admin/queue?status=pending")
            .with_api_headers()
            .with_state(admin.clone())
            .run_async(&app)
            .await
            .response_json()
            .await;

        let failed: Vec<QueueItem> = get("/api/admin/queue?status=failed")
            .with_api_headers()
            .with_state(admin)
            .run_async(&app)
            .await
            .response_json()
            .await;

        assert!(success.iter().all(|item| item.status == JobStatus::Success));
        assert_eq!(success.len(), 1);
        assert!(pending.iter().all(|item| item.status == JobStatus::Pending));
        assert_eq!(pending.len(), 1);
        assert_eq!(failed.len(), 0);
        Ok(())
    }
}
