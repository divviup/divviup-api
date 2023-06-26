use crate::{
    entity::{validators::base64, Account},
    handler::Error,
};
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

use super::{ActiveModel, Role};

#[derive(Deserialize, Serialize, Validate, Debug, Clone, Default)]
pub struct NewAggregator {
    #[validate(required, custom = "validate_role")]
    pub role: Option<String>,
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
    #[validate(url)]
    pub api_url: Option<String>,
    #[validate(required, url)]
    pub dap_url: Option<String>,
    #[validate(custom = "base64")]
    pub bearer_token: Option<String>,
}

fn validate_role(role: &str) -> Result<(), ValidationError> {
    Role::from_str(role)
        .map_err(|_| ValidationError::new("role"))
        .map(|_| ())
}

impl NewAggregator {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let errors = Validate::validate(self);
        match (&self.dap_url, &self.bearer_token) {
            (Some(_), Some(_)) | (None, None) => errors,
            (Some(_), None) => {
                let mut err = errors.err().unwrap_or_default();
                err.add("bearer_token", ValidationError::new("required"));
                Err(err)
            }

            (None, Some(_)) => {
                let mut err = errors.err().unwrap_or_default();
                err.add("dap_url", ValidationError::new("required"));
                Err(err)
            }
        }
    }

    pub fn build(self, account: Option<&Account>) -> Result<ActiveModel, Error> {
        self.validate()?;
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
        Ok(ActiveModel {
            role: Set(self.role.unwrap().parse().unwrap()),
            name: Set(self.name.unwrap()),
            api_url: Set(self.api_url.map(|u| u.parse()).transpose()?),
            dap_url: Set(self.dap_url.unwrap().parse()?),
            bearer_token: Set(self.bearer_token),
            id: Set(Uuid::new_v4()),
            account_id: Set(account.map(|account| account.id)),
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            deleted_at: Set(None),
            is_first_party: Set(account.is_none()),
        })
    }
}
