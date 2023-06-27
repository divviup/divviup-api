use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveModel};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateTask {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
}

impl UpdateTask {
    pub fn build(self, model: super::Model) -> Result<super::ActiveModel, crate::handler::Error> {
        self.validate()?;
        let mut am = model.into_active_model();
        am.name = ActiveValue::Set(self.name.unwrap());
        am.updated_at = ActiveValue::Set(TimeDateTimeWithTimeZone::now_utc());
        Ok(am)
    }
}
