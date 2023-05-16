use crate::{
    entity::*,
    queue::job::{
        v1::{reset_password::ResetPassword, V1},
        Job, JobError, SharedJobState,
    },
};
use sea_orm::{ConnectionTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct CreateUser {
    pub membership_id: Uuid,
}

impl CreateUser {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<Job>, JobError> {
        let membership = Memberships::find_by_id(self.membership_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                JobError::MissingRecord(String::from("membership"), self.membership_id.to_string())
            })?;

        let user_id = job_state
            .auth0_client
            .create_user(&membership.user_email)
            .await?;
        Ok(Some(Job::from(ResetPassword {
            membership_id: self.membership_id,
            user_id,
        })))
    }
}

impl From<CreateUser> for Job {
    fn from(value: CreateUser) -> Self {
        Self::V1(V1::CreateUser(value))
    }
}

impl PartialEq<Job> for CreateUser {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::CreateUser(c)) if c == self)
    }
}
impl PartialEq<CreateUser> for Job {
    fn eq(&self, other: &CreateUser) -> bool {
        matches!(self, Job::V1(V1::CreateUser(j)) if j == other)
    }
}
