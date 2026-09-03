use super::random_chars;
use crate::{
    clients::aggregator_client::api_types::{
        AggregatorApiConfig, AggregatorVdaf, AuthenticationToken, HpkeAeadId, HpkeConfig,
        HpkeKdfId, HpkeKemId, HpkePublicKey, JanusDuration, QueryType, Role, TaskCreate, TaskId,
        TaskIds, TaskPatch, TaskResponse, TaskUploadMetrics,
    },
    entity::aggregator::{Feature, Features},
};
use axum::{
    extract::{Path, Query, Request},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::random;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::iter::repeat_with;
use uuid::Uuid;

pub const BAD_BEARER_TOKEN: &str = "badbearertoken";

pub fn mock() -> Router {
    Router::new()
        .route("/", routing::get(get_config))
        .route("/tasks", routing::post(post_task))
        .route(
            "/tasks/{task_id}",
            routing::get(get_task).patch(patch_task).delete(delete_task),
        )
        .route("/task_ids", routing::get(task_ids))
        .route(
            "/tasks/{task_id}/metrics/uploads",
            routing::get(get_task_upload_metrics),
        )
        .layer(middleware::from_fn(bearer_token_check))
}

async fn bearer_token_check(request: Request, next: Next) -> Response {
    let token_is_valid = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| match s.split_once(' ') {
            Some(("Bearer", BAD_BEARER_TOKEN)) => false,
            Some(("Bearer", _)) => true,
            _ => false,
        });

    if token_is_valid {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn get_config() -> Json<AggregatorApiConfig> {
    Json(AggregatorApiConfig {
        dap_url: format!("https://dap.{}.example", random_chars(5))
            .parse()
            .unwrap(),
        role: random(),
        vdafs: Default::default(),
        query_types: Default::default(),
        protocol: random(),
        features: Features::from_iter([Feature::TokenHash]),
    })
}

async fn get_task_upload_metrics() -> Json<TaskUploadMetrics> {
    Json(TaskUploadMetrics {
        interval_collected: fastrand::u64(..1000),
        report_decode_failure: fastrand::u64(..1000),
        report_decrypt_failure: fastrand::u64(..1000),
        report_expired: fastrand::u64(..1000),
        report_outdated_key: fastrand::u64(..1000),
        report_success: fastrand::u64(..1000),
        report_too_early: fastrand::u64(..1000),
        report_duplicate_extension: fastrand::u64(..1000),
        task_expired: fastrand::u64(..1000),
    })
}

async fn get_task(Path(task_id): Path<String>) -> Json<TaskResponse> {
    Json(TaskResponse {
        task_id: task_id.parse().unwrap(),
        peer_aggregator_endpoint: "https://_".parse().unwrap(),
        query_type: QueryType::TimeInterval,
        vdaf: AggregatorVdaf::Prio3Count,
        role: Role::Leader,
        vdaf_verify_key: random_chars(10),
        max_batch_query_count: 100,
        task_expiration: None,
        report_expiry_age: None,
        min_batch_size: 1000,
        time_precision: JanusDuration::from_seconds(60),
        tolerable_clock_skew: JanusDuration::from_seconds(60),
        collector_hpke_config: random_hpke_config(),
        aggregator_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        collector_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        aggregator_hpke_configs: repeat_with(random_hpke_config).take(5).collect(),
    })
}

async fn post_task(Json(task_create): Json<TaskCreate>) -> Json<TaskResponse> {
    Json(task_response(task_create))
}

async fn patch_task(
    Path(task_id): Path<String>,
    Json(patch): Json<TaskPatch>,
) -> Json<TaskResponse> {
    Json(TaskResponse {
        task_id: task_id.parse().unwrap(),
        peer_aggregator_endpoint: "https://_".parse().unwrap(),
        query_type: QueryType::TimeInterval,
        vdaf: AggregatorVdaf::Prio3Count,
        role: Role::Leader,
        vdaf_verify_key: random_chars(10),
        max_batch_query_count: 100,
        task_expiration: patch.task_expiration,
        report_expiry_age: None,
        min_batch_size: 1000,
        time_precision: JanusDuration::from_seconds(60),
        tolerable_clock_skew: JanusDuration::from_seconds(60),
        collector_hpke_config: random_hpke_config(),
        aggregator_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        collector_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        aggregator_hpke_configs: repeat_with(random_hpke_config).take(5).collect(),
    })
}

async fn delete_task() -> StatusCode {
    StatusCode::OK
}

pub fn task_response(task_create: TaskCreate) -> TaskResponse {
    let task_id = TaskId::try_from(
        Sha256::digest(URL_SAFE_NO_PAD.decode(task_create.vdaf_verify_key).unwrap()).as_slice(),
    )
    .unwrap();
    TaskResponse {
        task_id,
        peer_aggregator_endpoint: task_create.peer_aggregator_endpoint,
        query_type: task_create.query_type,
        vdaf: task_create.vdaf,
        role: task_create.role,
        vdaf_verify_key: random_chars(10),
        max_batch_query_count: task_create.max_batch_query_count,
        task_expiration: task_create.task_expiration,
        report_expiry_age: None,
        min_batch_size: task_create.min_batch_size,
        time_precision: JanusDuration::from_seconds(task_create.time_precision),
        tolerable_clock_skew: JanusDuration::from_seconds(60),
        collector_hpke_config: random_hpke_config(),
        aggregator_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        collector_auth_token: Some(AuthenticationToken::new(random_chars(32))),
        aggregator_hpke_configs: repeat_with(random_hpke_config).take(5).collect(),
    }
}

pub fn random_hpke_config() -> HpkeConfig {
    HpkeConfig::new(
        random(),
        HpkeKemId::P256HkdfSha256,
        HpkeKdfId::HkdfSha512,
        HpkeAeadId::Aes256Gcm,
        HpkePublicKey::from(repeat_with(random::<u8>).take(32).collect::<Vec<_>>()),
    )
}

#[derive(Deserialize)]
struct TaskIdsQuery {
    pagination_token: Option<String>,
}

async fn task_ids(Query(query): Query<TaskIdsQuery>) -> Json<TaskIds> {
    match query.pagination_token.as_deref() {
        None => Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("second".into()),
        }),

        Some("second") => Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("last".into()),
        }),

        _ => Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string()).take(5).collect(),
            pagination_token: None,
        }),
    }
}
