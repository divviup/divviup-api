use crate::clients::aggregator_client::api_types::{
    HpkeAeadId, HpkeConfig, HpkeKdfId, HpkeKemId, HpkePublicKey, JanusDuration, TaskCreate, TaskId,
    TaskIds, TaskMetrics, TaskResponse,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use fastrand::alphanumeric;
use querystrong::QueryStrong;
use rand::random;
use sha2::{Digest, Sha256};
use std::iter::repeat_with;
use trillium::{Conn, Handler, Status};
use trillium_api::{api, Json};
use trillium_logger::{dev_formatter, logger};
use trillium_router::router;
use uuid::Uuid;

pub fn aggregator_api() -> impl Handler {
    (
        logger().with_formatter(("[aggregator mock] ", dev_formatter)),
        router()
            .post("/tasks", api(post_task))
            .get("/task_ids", api(task_ids))
            .delete("/tasks/:task_id", Status::Ok)
            .get("/tasks/:task_id/metrics", api(get_task_metrics)),
    )
}

async fn get_task_metrics(_: &mut Conn, (): ()) -> Json<TaskMetrics> {
    Json(TaskMetrics {
        reports: fastrand::u64(..1000),
        report_aggregations: fastrand::u64(..1000),
    })
}

async fn post_task(_: &mut Conn, Json(task_create): Json<TaskCreate>) -> Json<TaskResponse> {
    Json(task_response(task_create))
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
        vdaf_verify_keys: vec![repeat_with(alphanumeric).take(10).collect()],
        max_batch_query_count: task_create.max_batch_query_count,
        task_expiration: task_create.task_expiration,
        report_expiry_age: None,
        min_batch_size: task_create.min_batch_size,
        time_precision: JanusDuration::from_seconds(task_create.time_precision),
        tolerable_clock_skew: JanusDuration::from_seconds(60),
        collector_hpke_config: random_hpke_config(),
        aggregator_auth_tokens: vec![],
        collector_auth_tokens: vec![],
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

async fn task_ids(conn: &mut Conn, (): ()) -> Result<Json<TaskIds>, Status> {
    let query = QueryStrong::parse(conn.querystring()).map_err(|_| Status::InternalServerError)?;
    match query.get_str("pagination_token") {
        None => Ok(Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("second".into()),
        })),

        Some("second") => Ok(Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("last".into()),
        })),

        _ => Ok(Json(TaskIds {
            task_ids: repeat_with(|| Uuid::new_v4().to_string()).take(5).collect(),
            pagination_token: None,
        })),
    }
}
