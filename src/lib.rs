#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]

pub mod clients;
mod config;
mod db;
#[macro_use]
pub mod entity;
mod crypter;
pub mod handler;
pub mod permissions;
pub mod queue;
mod routes;
pub mod telemetry;
mod user;

pub use config::{Config, ConfigError};
pub use crypter::Crypter;
pub use db::Db;
pub use handler::{custom_mime_types::CONTENT_TYPE, DivviupApi, Error};
pub use permissions::{Permissions, PermissionsActor};
pub use queue::Queue;
pub use routes::routes;
pub use user::{User, USER_SESSION_KEY};

#[cfg(test)]
pub mod test;

pub mod api_mocks;
