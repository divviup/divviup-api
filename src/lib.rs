#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]

pub mod aggregator_api_mock;
pub mod auth0_client;
pub mod client;
mod config;
mod db;
pub mod entity;
pub(crate) mod handler;
pub mod postmark_client;
mod routes;
pub mod telemetry;
mod user;

pub use client::AggregatorClient;
pub use config::{ApiConfig, ApiConfigError};
pub use db::Db;
pub use handler::DivviupApi;
pub use routes::routes;
pub use user::{User, USER_SESSION_KEY};
