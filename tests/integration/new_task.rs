use divviup_api::entity::aggregator::{Feature, Features};
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

pub async fn assert_no_errors(app: &DivviupApi, new_task: &mut NewTask, field: &str) {
    let account = fixtures::account(app).await;
    let errors = new_task
        .normalize_and_validate(account, app.db())
        .await
        .unwrap_err();
    let errors = errors
        .field_errors()
        .get(field)
        .map(|c| c.iter().map(|error| &error.code).collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(errors.is_empty(), "{errors:?}");
}

pub async fn assert_errors_nested(
    app: &DivviupApi,
    account: Account,
    new_task: &mut NewTask,
    expected_errors: Value,
) {
    let errors = new_task
        .normalize_and_validate(account, app.db())
        .await
        .unwrap_err();
    let serialized = serde_json::to_value(errors).unwrap();
    assert_eq!(serialized, expected_errors);
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
async fn time_bucketed_fixed_size(app: DivviupApi) -> TestResult {
    let mut leader = fixtures::aggregator(&app, None).await.into_active_model();
    leader.role = ActiveValue::Set(Role::Leader);
    leader.features =
        ActiveValue::Set(Features::from_iter([Feature::TimeBucketedFixedSize]).into());
    let leader = leader.update(app.db()).await?;

    let mut helper = fixtures::aggregator(&app, None).await.into_active_model();
    helper.role = ActiveValue::Set(Role::Helper);
    let helper = helper.update(app.db()).await?;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            max_batch_size: None,
            batch_time_window_size_seconds: Some(300),
            ..Default::default()
        },
        "batch_time_window_size_seconds",
        &["missing-max-batch-size"],
    )
    .await;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(123),
            min_batch_size: Some(100),
            max_batch_size: Some(100),
            batch_time_window_size_seconds: Some(300),
            ..Default::default()
        },
        "batch_time_window_size_seconds",
        &["not-multiple-of-time-precision"],
    )
    .await;

    assert_no_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(123),
            min_batch_size: Some(100),
            max_batch_size: Some(100),
            batch_time_window_size_seconds: Some(300),
            ..Default::default()
        },
        "leader_aggregator_id",
    )
    .await;

    let mut leader = fixtures::aggregator(&app, None).await.into_active_model();
    leader.role = ActiveValue::Set(Role::Leader);
    let leader = leader.update(app.db()).await?;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            min_batch_size: Some(100),
            max_batch_size: Some(100),
            batch_time_window_size_seconds: Some(300),
            ..Default::default()
        },
        "leader_aggregator_id",
        &["time-bucketed-fixed-size-unsupported"],
    )
    .await;

    Ok(())
}

#[test(harness = set_up)]
async fn pure_dp_discrete_laplace(app: DivviupApi) -> TestResult {
    let account = fixtures::account(&app).await;
    let collector_credential = fixtures::collector_credential(&app, &account).await;

    let mut leader = fixtures::aggregator(&app, None).await.into_active_model();
    leader.role = ActiveValue::Set(Role::Leader);
    leader.features =
        ActiveValue::Set(Features::from_iter([Feature::PureDpDiscreteLaplace]).into());
    let leader = leader.update(app.db()).await?;

    let mut helper = fixtures::aggregator(&app, None).await.into_active_model();
    helper.role = ActiveValue::Set(Role::Helper);
    helper.features =
        ActiveValue::Set(Features::from_iter([Feature::PureDpDiscreteLaplace]).into());
    let helper = helper.update(app.db()).await?;

    assert_errors_nested(
        &app,
        account.clone(),
        &mut NewTask {
            name: Some("test".into()),
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            min_batch_size: Some(100),
            collector_credential_id: Some(collector_credential.id.to_string()),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::NoDifferentialPrivacy,
                        budget: task::vdaf::DpBudget {
                            epsilon: Some(Vec::from([Vec::from([1]), Vec::from([1])])),
                        },
                    },
                },
            ))),
            ..Default::default()
        },
        json!({
            "vdaf": {
                "dp_strategy": [{
                    "code": "extra_epsilon",
                    "message": null,
                    "params": {
                        "value": {
                            "dp_strategy": "NoDifferentialPrivacy",
                            "budget": {
                                "epsilon": [[1], [1]],
                            },
                        },
                    },
                }],
            },
        }),
    )
    .await;
    assert_errors_nested(
        &app,
        account.clone(),
        &mut NewTask {
            name: Some("test".into()),
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            min_batch_size: Some(100),
            collector_credential_id: Some(collector_credential.id.to_string()),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::PureDpDiscreteLaplace,
                        budget: task::vdaf::DpBudget { epsilon: None },
                    },
                },
            ))),
            ..Default::default()
        },
        json!({
            "vdaf": {
                "dp_strategy": [{
                    "code": "missing_epsilon",
                    "message": null,
                    "params": {
                        "value": {
                            "dp_strategy": "PureDpDiscreteLaplace",
                            "budget": {
                                "epsilon": null,
                            },
                        },
                    },
                }],
            },
        }),
    )
    .await;
    assert_errors_nested(
        &app,
        account.clone(),
        &mut NewTask {
            name: Some("test".into()),
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            min_batch_size: Some(100),
            collector_credential_id: Some(collector_credential.id.to_string()),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::PureDpDiscreteLaplace,
                        budget: task::vdaf::DpBudget {
                            epsilon: Some(Vec::from([Vec::from([u32::MAX, u32::MAX])])),
                        },
                    },
                },
            ))),
            ..Default::default()
        },
        json!({
            "vdaf": {
                "dp_strategy": {
                    "budget": {
                        "epsilon": [{
                            "code": "length",
                            "message": null,
                            "params": {
                                "equal": 2,
                                "value": [[u32::MAX, u32::MAX]],
                            },
                        }],
                    },
                },
            },
        }),
    )
    .await;

    assert_no_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::PureDpDiscreteLaplace,
                        budget: task::vdaf::DpBudget {
                            epsilon: Some(Vec::from([Vec::from([1]), Vec::from([1])])),
                        },
                    },
                },
            ))),
            ..Default::default()
        },
        "leader_aggregator_id",
    )
    .await;

    let mut plain_leader = fixtures::aggregator(&app, None).await.into_active_model();
    plain_leader.role = ActiveValue::Set(Role::Leader);
    let plain_leader = plain_leader.update(app.db()).await?;

    let mut plain_helper = fixtures::aggregator(&app, None).await.into_active_model();
    plain_helper.role = ActiveValue::Set(Role::Helper);
    let plain_helper = plain_helper.update(app.db()).await?;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(plain_leader.id.to_string()),
            helper_aggregator_id: Some(helper.id.to_string()),
            time_precision_seconds: Some(300),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::PureDpDiscreteLaplace,
                        budget: task::vdaf::DpBudget {
                            epsilon: Some(Vec::from([Vec::from([1]), Vec::from([1])])),
                        },
                    },
                },
            ))),
            ..Default::default()
        },
        "leader_aggregator_id",
        &["pure-dp-discrete-laplace-unsupported"],
    )
    .await;

    assert_errors(
        &app,
        &mut NewTask {
            leader_aggregator_id: Some(leader.id.to_string()),
            helper_aggregator_id: Some(plain_helper.id.to_string()),
            time_precision_seconds: Some(300),
            vdaf: Some(task::vdaf::Vdaf::Histogram(task::vdaf::Histogram::Opaque(
                task::vdaf::BucketLength {
                    length: 10,
                    chunk_length: Some(3),
                    dp_strategy: task::vdaf::DpStrategy {
                        dp_strategy: task::vdaf::DpStrategyKind::PureDpDiscreteLaplace,
                        budget: task::vdaf::DpBudget {
                            epsilon: Some(Vec::from([Vec::from([1]), Vec::from([1])])),
                        },
                    },
                },
            ))),
            ..Default::default()
        },
        "helper_aggregator_id",
        &["pure-dp-discrete-laplace-unsupported"],
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
