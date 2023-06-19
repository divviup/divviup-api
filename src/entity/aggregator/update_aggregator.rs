use sea_orm::{ActiveValue::Set, IntoActiveModel};
use serde::Deserialize;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
pub struct UpdateAggregator {
    #[validate(required, length(min = 1))]
    pub name: Option<String>,
}

impl UpdateAggregator {
    pub fn build(self, model: super::Model) -> Result<super::ActiveModel, crate::handler::Error> {
        self.validate()?;
        let mut am = model.into_active_model();
        am.name = Set(self.name.unwrap());
        am.updated_at = Set(OffsetDateTime::now_utc());
        Ok(am)
    }
}
