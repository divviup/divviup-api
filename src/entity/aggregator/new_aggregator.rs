use crate::{
    entity::{url_safe_base64, Account},
    handler::Error,
};
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::ActiveModel;

#[derive(Deserialize, Serialize, Validate, Debug, Clone, Default)]
pub struct NewAggregator {
    #[validate(required, custom = "validate_role")]
    pub role: Option<String>,
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
    #[validate(required, url)]
    pub api_url: Option<String>,
    #[validate(required, url)]
    pub dap_url: Option<String>,
    #[validate(required, custom = "url_safe_base64")]
    pub bearer_token: Option<String>,
}

fn validate_role(role: &str) -> Result<(), ValidationError> {
    super::Role::from_str(role)
        .map_err(|_| ValidationError::new("role"))
        .map(|_| ())
}

impl NewAggregator {
    pub fn build(self, account: Option<&Account>) -> Result<ActiveModel, Error> {
        self.validate()?;

        Ok(ActiveModel {
            role: Set(self.role.unwrap().parse().unwrap()),
            name: Set(self.name.unwrap()),
            api_url: Set(self.api_url.unwrap().parse()?),
            dap_url: Set(self.dap_url.unwrap().parse()?),
            bearer_token: Set(self.bearer_token.unwrap()),
            id: Set(Uuid::new_v4()),
            account_id: Set(account.map(|account| account.id)),
            created_at: Set(OffsetDateTime::now_utc()),
            updated_at: Set(OffsetDateTime::now_utc()),
            deleted_at: Set(None),
        })
    }
}
