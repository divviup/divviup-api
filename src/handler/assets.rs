use std::time::Duration;

use crate::ApiConfig;
use trillium::{Conn, Handler, KnownHeaderName::CacheControl, Status};
use trillium_caching_headers::CacheControlDirective::{MaxAge, NoCache};
use trillium_router::RouterConnExt;
use trillium_static_compiled::{static_compiled, StaticCompiledHandler};

struct HostFilter<H: Handler> {
    handler: H,
    host: String,
}

#[trillium::async_trait]
impl<H: Handler> Handler for HostFilter<H> {
    async fn run(&self, conn: Conn) -> Conn {
        if Some(&*self.host) == conn.inner().host() {
            self.handler.run(conn).await
        } else {
            conn
        }
    }

    async fn before_send(&self, conn: Conn) -> Conn {
        if Some(&*self.host) == conn.inner().host() {
            self.handler.before_send(conn).await
        } else {
            conn
        }
    }
}

pub fn static_assets(config: &ApiConfig) -> impl Handler {
    HostFilter {
        handler: ReactApp(static_compiled!("$OUT_DIR").with_index_file("index.html")),
        host: config.app_url.host().unwrap().to_string(),
    }
}

struct ReactApp(StaticCompiledHandler);
#[trillium::async_trait]
impl Handler for ReactApp {
    async fn run(&self, mut conn: Conn) -> Conn {
        conn = self.0.run(conn).await;
        if conn.is_halted() {
            // https://create-react-app.dev/docs/production-build
            if conn.path().starts_with("/static") {
                conn.with_header(
                    CacheControl,
                    MaxAge(Duration::from_secs(60 * 60 * 24 * 365)),
                )
            } else {
                conn.with_header(CacheControl, NoCache)
            }
        } else {
            conn
        }
    }

    async fn before_send(&self, mut conn: Conn) -> Conn {
        if !conn.is_halted() && conn.route().is_none() {
            conn.push_path("/index.html".into());
            conn = self
                .0
                .run(conn)
                .await
                .with_header(CacheControl, NoCache)
                .with_status(Status::Ok);
            conn.pop_path();
        }

        self.0.before_send(conn).await
    }
}
