use test_support::{assert_eq, test, *};

pub async fn assert_errors(app: &DivviupApi, new_task: &mut NewTask, field: &str, codes: &[&str]) {
    let account = fixtures::account(app).await;
    assert_eq!(
        new_task
            .normalize_and_validate(account, app.db())
            .await
            .unwrap_err()
            .field_errors()
            .get(field)
            .map(|c| c.iter().map(|error| &error.code).collect::<Vec<_>>())
            .unwrap_or_default(),
        codes
    );
}

#[test(harness = set_up)]
async fn batch_size(app: DivviupApi) -> TestResult {
    assert_errors(
        &app,
        &mut NewTask {
            min_batch_size: Some(100),
            max_batch_size: Some(50),
            ..Default::default()
        },
        "min_batch_size",
        &["min_greater_than_max"],
    )
    .await;

    assert_errors(
        &app,
        &mut NewTask {
            min_batch_size: Some(100),
            max_batch_size: Some(50),
            ..Default::default()
        },
        "max_batch_size",
        &["min_greater_than_max"],
    )
    .await;
    Ok(())
}

#[test(harness = set_up)]
async fn aggregator_roles(app: DivviupApi) -> TestResult {
    let mut leader = fixtures::aggregator(&app, None).await.into_active_model();
    leader.role = ActiveValue::Set(Role::Leader);
    let leader = leader.update(app.db()).await?;

    let mut helper = fixtures::aggregator(&app, None).await.into_active_model();
    helper.role = ActiveValue::Set(Role::Helper);
    let helper = helper.update(app.db()).await?;

    let either = fixtures::aggregator(&app, None).await;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(helper.id.to_string()),
            helper_aggregator_id: Some(either.id.to_string()),
            ..Default::default()
        },
        "leader_aggregator_id",
        &["role"],
    )
    .await;

    assert_errors(
        &app,
        &mut NewTask {
            helper_aggregator_id: Some(leader.id.to_string()),
            leader_aggregator_id: Some(either.id.to_string()),
            ..Default::default()
        },
        "helper_aggregator_id",
        &["role"],
    )
    .await;

    let mut ok_aggregators = NewTask {
        helper_aggregator_id: Some(helper.id.to_string()),
        leader_aggregator_id: Some(leader.id.to_string()),
        ..Default::default()
    };

    assert_errors(&app, &mut ok_aggregators, "helper_aggregator_id", &[]).await;
    assert_errors(&app, &mut ok_aggregators, "leader_aggregator_id", &[]).await;
    Ok(())
}
