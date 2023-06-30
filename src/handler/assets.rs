use crate::{handler::origin_router, ApiConfig};
use std::time::Duration;
use trillium::{
    Conn, Handler,
    KnownHeaderName::{self, CacheControl},
    Status,
};
use trillium_caching_headers::CacheControlDirective::{MaxAge, NoCache};
use trillium_static_compiled::{static_compiled, StaticCompiledHandler};
use url::Url;

const ONE_YEAR: Duration = Duration::from_secs(60 * 60 * 24 * 365);

pub fn static_assets(config: &ApiConfig) -> impl Handler {
    origin_router().with_handler(
        config.app_url.as_ref(),
        ReactApp {
            handler: static_compiled!("$ASSET_DIR").with_index_file("index.html"),
            api_url: config.api_url.clone(),
        },
    )
}

struct ReactApp {
    handler: StaticCompiledHandler,
    api_url: Url,
}

#[trillium::async_trait]
impl Handler for ReactApp {
    async fn run(&self, mut conn: Conn) -> Conn {
        if conn.path().starts_with("/api_url") {
            return conn
                .ok(self.api_url.to_string())
                .with_header(KnownHeaderName::ContentType, "text/plain")
                .with_header(CacheControl, NoCache);
        }

        conn = self.handler.run(conn).await;

        if conn.is_halted() {
            if conn.path().starts_with("/assets" /*hashed assets*/) {
                conn.with_header(CacheControl, MaxAge(ONE_YEAR))
            } else {
                conn.with_header(CacheControl, NoCache)
            }
        } else {
            conn.push_path("/index.html".into());
            conn = self
                .handler
                .run(conn)
                .await
                .with_header(CacheControl, NoCache)
                .with_status(Status::Ok);
            conn.pop_path();
            conn
        }
    }
}
