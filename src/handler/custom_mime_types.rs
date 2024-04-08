pub const CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

pub struct ReplaceMimeTypes;

use trillium::{
    Conn, Handler,
    KnownHeaderName::{Accept, ContentType},
    Status::{NotAcceptable, UnsupportedMediaType},
};

#[trillium::async_trait]
impl Handler for ReplaceMimeTypes {
    async fn run(&self, mut conn: Conn) -> Conn {
        let request_headers = conn.inner_mut().request_headers_mut();
        if let Some(CONTENT_TYPE) | None = request_headers.get_str(ContentType) {
            request_headers.insert(ContentType, "application/json");
        } else {
            return conn.with_status(UnsupportedMediaType).halt();
        }

        if Some(CONTENT_TYPE) == request_headers.get_str(Accept) {
            request_headers.insert(Accept, "application/json");
        } else {
            return conn.with_status(NotAcceptable).halt();
        }

        conn
    }

    async fn before_send(&self, conn: Conn) -> Conn {
        conn.with_response_header(ContentType, CONTENT_TYPE)
    }
}
