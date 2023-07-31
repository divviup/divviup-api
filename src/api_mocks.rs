use crate::handler::origin_router;
use trillium::Handler;
use trillium_http::KnownHeaderName;
use trillium_logger::{
    formatters::{method, request_header, status, url},
    logger,
};
use trillium_macros::Handler;

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
                request_header(KnownHeaderName::Host),
                " ",
                url,
                " ",
                status,
            )),
            origin_router()
                .with_handler(postmark_url, postmark::mock())
                .with_handler(auth0_url, auth0::mock(auth0_url)),
            aggregator_api::mock(),
        )))
    }
}
