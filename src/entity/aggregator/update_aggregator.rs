use crate::{
    clients::{AggregatorClient, ClientError},
    entity::Aggregator,
    Crypter, Error,
};
use sea_orm::{ActiveValue, IntoActiveModel};
use serde::Deserialize;
use time::OffsetDateTime;
use trillium_client::Client;
use trillium_http::Status;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateAggregator {
    #[validate(length(min = 1))]
    pub name: Option<String>,
    pub bearer_token: Option<String>,
}

impl UpdateAggregator {
    pub async fn build(
        self,
        aggregator: Aggregator,
        client: Client,
        crypter: &Crypter,
    ) -> Result<super::ActiveModel, Error> {
        self.validate()?;
        let api_url = aggregator.api_url.clone().into();

        let bearer_token = match self.bearer_token {
            Some(bearer_token) => bearer_token,
            None => aggregator.bearer_token(crypter)?,
        };

        let mut aggregator = aggregator.into_active_model();
        if let Some(name) = self.name {
            aggregator.name = ActiveValue::Set(name);
        }
        let aggregator_config = AggregatorClient::get_config(client, api_url, &bearer_token)
            .await
            .map_err(|e| match e {
                ClientError::HttpStatusNotSuccess {
                    status: Some(Status::Unauthorized | Status::Forbidden),
                    ..
                } => {
                    let mut validation_errors = ValidationErrors::new();
                    validation_errors
                        .add("bearer_token", ValidationError::new("token-not-recognized"));
                    validation_errors.into()
                }

                other => Error::from(other),
            })?;

        aggregator.query_types = ActiveValue::Set(aggregator_config.query_types.into());
        aggregator.vdafs = ActiveValue::Set(aggregator_config.vdafs.into());
        aggregator.features = ActiveValue::Set(aggregator_config.features.into());
        aggregator.encrypted_bearer_token = ActiveValue::Set(crypter.encrypt(
            aggregator.api_url.as_ref().as_ref().as_bytes(),
            bearer_token.as_bytes(),
        )?);
        aggregator.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        Ok(aggregator)
    }
}
