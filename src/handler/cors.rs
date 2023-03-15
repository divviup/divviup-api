use crate::ApiConfig;
use std::sync::Arc;
use trillium::{
    Conn,
    KnownHeaderName::{
        AccessControlAllowCredentials, AccessControlAllowHeaders, AccessControlAllowMethods,
        AccessControlAllowOrigin, AccessControlMaxAge, Origin,
    },
    Method, Status,
};

pub async fn cors_headers(mut conn: Conn) -> Conn {
    let config = conn.state::<Arc<ApiConfig>>().unwrap();
    let mut origin = config.app_url.to_string();
    origin.pop();

    let conn_origin = conn.headers().get_str(Origin);
    if conn_origin == Some(&origin) {
        conn.headers_mut().extend([
            (
                AccessControlAllowMethods,
                "POST, DELETE, OPTIONS, GET, PATCH",
            ),
            (AccessControlAllowCredentials, "true"),
            (
                AccessControlAllowHeaders,
                "Content-Type, If-None-Match, If-Modified-Since, Etag",
            ),
            (AccessControlMaxAge, "86400"),
        ]);
        conn.headers_mut().append(AccessControlAllowOrigin, origin);

        if conn.method() == Method::Options {
            return conn.with_status(Status::NoContent).halt();
        }
    }
    conn
}
