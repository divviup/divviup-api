use crate::ApiConfig;
use trillium::{
    Conn, Handler,
    KnownHeaderName::{
        AccessControlAllowCredentials, AccessControlAllowHeaders, AccessControlAllowMethods,
        AccessControlAllowOrigin, AccessControlMaxAge, Origin,
    },
    Method, Status,
};

#[derive(Debug)]
struct CorsHeaders {
    origin: String,
}

#[trillium::async_trait]
impl Handler for CorsHeaders {
    async fn run(&self, mut conn: Conn) -> Conn {
        let conn_origin = conn.headers().get_str(Origin);

        if conn_origin == Some(&self.origin) {
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
            conn.headers_mut()
                .insert(AccessControlAllowOrigin, self.origin.clone());

            if conn.method() == Method::Options {
                return conn.with_status(Status::NoContent).halt();
            }
        }
        conn
    }
}

impl CorsHeaders {
    pub fn new(config: &ApiConfig) -> Self {
        let mut origin = config.app_url.to_string();
        origin.pop();
        Self { origin }
    }
}

pub fn cors_headers(config: &ApiConfig) -> impl Handler {
    CorsHeaders::new(config)
}
