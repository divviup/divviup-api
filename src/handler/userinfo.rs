use crate::handler::oauth2::OauthClient;
use crate::ApiConfig;
use std::sync::Arc;
use trillium::{
    Conn,
    KnownHeaderName::{self, Location},
    Status,
};
use trillium_sessions::SessionConnExt;

pub(crate) async fn userinfo(conn: Conn) -> Conn {
    conn.ok(body)
}
