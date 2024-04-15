use crate::{
    clients::aggregator_client::api_types::TaskResponse,
    entity::{
        Aggregator, AggregatorColumn, Aggregators, CollectorCredentialColumn, CollectorCredentials,
    },
};
use serde::Deserialize;
use std::fmt::Debug;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use validator::{Validate, ValidationError};

pub mod vdaf;
use vdaf::Vdaf;
mod new_task;
pub use new_task::NewTask;
mod update_task;
pub use update_task::UpdateTask;
mod provisionable_task;
pub use provisionable_task::ProvisionableTask;
pub mod model;
pub use model::*;

pub const DEFAULT_EXPIRATION_DURATION: Duration = Duration::days(365);

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum TaskProvisioningError {
    #[error("discrepancy in {0}")]
    Discrepancy(&'static str),
}

pub(crate) fn assert_same<T: Eq + Debug>(
    ours: T,
    theirs: T,
    property: &'static str,
) -> Result<(), TaskProvisioningError> {
    if ours == theirs {
        Ok(())
    } else {
        log::error!("{property} discrepancy. ours: {ours:?}, theirs: {theirs:?}");
        Err(TaskProvisioningError::Discrepancy(property))
    }
}
