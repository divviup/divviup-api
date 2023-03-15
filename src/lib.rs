#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]

mod config;
mod db;
pub(crate) mod entity;
pub(crate) mod handler;
mod routes;
mod user;

pub use config::{ApiConfig, ApiConfigError};
pub use db::{Db, DbConnExt};
pub use handler::divviup_api;
pub use routes::routes;
pub use user::{User, USER_SESSION_KEY};
