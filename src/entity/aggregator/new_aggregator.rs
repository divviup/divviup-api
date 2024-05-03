use super::ActiveModel;
use crate::{
    clients::{AggregatorClient, ClientError},
    entity::{url::Url, Account, Aggregator},
    handler::Error,
};
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use trillium_client::Client;
use trillium_http::Status;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Serialize, Validate, Debug, Clone, Default)]
pub struct NewAggregator {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
    #[cfg_attr(
        not(feature = "integration-testing"),
        validate(custom(function = "https"))
    )]
    pub api_url: Option<String>,
    pub bearer_token: Option<String>,
    pub is_first_party: Option<bool>,
}

#[cfg_attr(feature = "integration-testing", allow(dead_code))]
fn https(url: &str) -> Result<(), ValidationError> {
    let url = url::Url::from_str(url).map_err(|_| ValidationError::new("https-url"))?;
    if url.scheme() != "https" {
        return Err(ValidationError::new("https-url"));
    }
    Ok(())
}

impl NewAggregator {
    pub async fn build(
        self,
        account: Option<&Account>,
        client: Client,
        crypter: &crate::Crypter,
    ) -> Result<ActiveModel, Error> {
        self.validate()?;
        let aggregator_config = AggregatorClient::get_config(
            client,
            self.api_url.as_ref().unwrap().parse()?,
            self.bearer_token.as_ref().unwrap(),
        )
        .await
        .map_err(|e| match e {
            ClientError::HttpStatusNotSuccess {
                status: Some(Status::Unauthorized | Status::Forbidden),
                ..
            } => {
                let mut ve = ValidationErrors::new();
                ve.add("bearer_token", ValidationError::new("token-not-recognized"));
                ve.into()
            }

            ClientError::Http(_)
            | ClientError::HttpStatusNotSuccess {
                status: Some(Status::NotFound),
                ..
            } => {
                let mut ve = ValidationErrors::new();
                ve.add("api_url", ValidationError::new("http-error"));
                ve.into()
            }

            other => Error::from(other),
        })?;

        // unwrap safety: the below unwraps will never panic, because
        // the above call to `NewAggregator::validate` will
        // early-return if any of the required `Option`s is `None`.
        //
        // This is an unfortunate consequence of the combination of
        // `serde` and `validate`, and would be resolved by a
        // potential deserializer-and-validator library that
        // accumulates errors instead of bailing on the first
        // error. As this deserialize-and-validate behavior is outside
        // of the scope of this repository, we work around this by
        // double-checking these Options -- once in validate, and
        // again in the conversion to non-optional fields.

        let api_url: Url = self.api_url.as_ref().unwrap().parse()?;
        let encrypted_bearer_token = crypter.encrypt(
            api_url.as_ref().as_bytes(),
            self.bearer_token.as_deref().unwrap_or_default().as_bytes(),
        )?;

        Ok(Aggregator {
            role: aggregator_config.role,
            name: self.name.unwrap(),
            api_url: self.api_url.unwrap().parse()?,
            dap_url: aggregator_config.dap_url.into(),
            encrypted_bearer_token,
            id: Uuid::new_v4(),
            account_id: account.map(|account| account.id),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
            is_first_party: account.is_none() && self.is_first_party.unwrap_or(true),
            query_types: aggregator_config.query_types.into(),
            vdafs: aggregator_config.vdafs.into(),
            protocol: aggregator_config.protocol,
            features: aggregator_config.features.into(),
        }
        .into_active_model())
    }
}
