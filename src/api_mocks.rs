use crate::handler::origin_router;
use trillium::{Conn, Handler};
use trillium_http::KnownHeaderName;
use trillium_logger::{
    formatters::{method, status, url},
    logger,
};
use trillium_macros::Handler;
use trillium_router::Router;

pub mod aggregator_api;
pub mod auth0;
pub mod postmark;

fn random_chars(n: usize) -> String {
    std::iter::repeat_with(fastrand::alphabetic)
        .take(n)
        .collect()
}

#[derive(Handler, Debug)]
pub struct ApiMocks(Box<dyn Handler>);

impl ApiMocks {
    pub fn new(postmark_url: &str, auth0_url: &str) -> Self {
        Self(Box::new((
            logger().with_formatter((
                "[mock] ",
                method,
                " ",
                move |conn: &Conn, _| {
                    conn.headers()
                        .get_str(KnownHeaderName::Host)
                        .unwrap_or_default()
                        .to_string()
                },
                " ",
                url,
                " ",
                status,
            )),
            origin_router()
                .with_handler(postmark_url, postmark::mock())
                .with_handler(auth0_url, auth0::mock(auth0_url)),
            // Add a path prefix before the aggregator API mock. Aggregator test fixtures must
            // include this prefix in their URLs.
            Router::new().all("prefix/*", aggregator_api::mock()),
        )))
    }
}
