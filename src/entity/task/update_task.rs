use janus_messages::Time as JanusTime;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::try_join;
use trillium_client::Client;
use validator::{Validate, ValidationError};

use crate::{deserialize_some, entity::Aggregator, handler::Error, Crypter, Db};

use super::assert_same;

#[derive(Default, Deserialize, Validate, Debug)]
pub struct UpdateTask {
    #[validate(custom(function = "validate_name"))]
    name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    expiration: Option<Expiration>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct Expiration(#[serde(default, with = "time::serde::rfc3339::option")] Option<OffsetDateTime>);

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::new("name-too-short"));
    }
    Ok(())
}

impl UpdateTask {
    pub async fn update_aggregator_expiration(
        &self,
        aggregator: Aggregator,
        task_id: &str,
        http_client: &Client,
        crypter: &Crypter,
    ) -> Result<(), Error> {
        let expiration = self.expiration.as_ref().unwrap().0.map(|expiration| {
            JanusTime::from_seconds_since_epoch(expiration.unix_timestamp().try_into().unwrap())
        });
        let response = aggregator
            .client(http_client.clone(), crypter)?
            .update_task_expiration(task_id, expiration)
            .await?;
        assert_same(expiration, response.task_expiration, "expiration")?;
        Ok(())
    }

    /// Validates the request. Updates task definitions in the aggregators, if necessary. Returns
    /// an [`ActiveModel`] for committing to the database.
    pub async fn update(
        self,
        http_client: &Client,
        db: &Db,
        crypter: &Crypter,
        model: super::Model,
    ) -> Result<super::ActiveModel, Error> {
        self.validate()?;
        let mut am = model.clone().into_active_model();
        if let Some(ref name) = self.name {
            am.name = ActiveValue::Set(name.clone());
        }
        if let Some(ref expiration) = self.expiration {
            try_join!(
                self.update_aggregator_expiration(
                    model.leader_aggregator(db).await?,
                    &model.id,
                    http_client,
                    crypter
                ),
                self.update_aggregator_expiration(
                    model.helper_aggregator(db).await?,
                    &model.id,
                    http_client,
                    crypter
                )
            )?;
            am.expiration = ActiveValue::set(expiration.0);
        }
        if am.is_changed() {
            am.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        }
        Ok(am)
    }

    pub fn expiration(expiration: Option<OffsetDateTime>) -> Self {
        Self {
            expiration: Some(Expiration(expiration)),
            ..Default::default()
        }
    }
}
