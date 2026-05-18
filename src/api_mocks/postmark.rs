use axum::{routing, Json, Router};
use serde_json::json;

pub fn mock() -> Router {
    Router::new().route(
        "/email/withTemplate",
        routing::post(|| async { Json(json!({})) }),
    )
}
