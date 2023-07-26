use serde_json::json;
use trillium::Handler;
use trillium_api::{Halt, Json};
use trillium_router::router;
pub fn mock() -> impl Handler {
    (router().post("/email/withTemplate", Json(json!({}))), Halt)
}
