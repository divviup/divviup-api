#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

pub mod clients;
mod config;
mod crypter;
mod db;
pub mod entity;
pub mod handler;
pub mod permissions;
pub mod queue;
mod routes;
pub mod telemetry;
pub mod trace;
mod user;

pub use config::{Config, ConfigError, FeatureFlags};
pub use crypter::Crypter;
pub use db::Db;
pub use handler::{custom_mime_types::CONTENT_TYPE, DivviupApi, Error};
pub use opentelemetry;
pub use permissions::{Permissions, PermissionsActor};
pub use queue::Queue;
pub use routes::routes;
use serde::{Deserialize, Deserializer};
pub use user::{User, USER_SESSION_KEY};

#[cfg(test)]
pub mod test;

pub mod api_mocks;

/// Any value that is present is considered Some value, including null.
/// See https://github.com/serde-rs/serde/issues/984#issuecomment-314143738
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
