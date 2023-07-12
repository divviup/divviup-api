use crate::entity::validators::base64;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateAggregator {
    #[validate(length(min = 1))]
    pub name: Option<String>,

    #[validate(custom = "base64", length(min = 8))]
    pub bearer_token: Option<String>,
}

impl UpdateAggregator {
    pub fn build(self, model: super::Model) -> Result<super::ActiveModel, crate::handler::Error> {
        self.validate()?;
        let mut am = model.into_active_model();
        if let Some(name) = self.name {
            am.name = ActiveValue::Set(name);
        }

        if let Some(bearer_token) = self.bearer_token {
            am.bearer_token = ActiveValue::Set(bearer_token);
        }

        if am.is_changed() {
            am.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        }
        Ok(am)
    }
}
