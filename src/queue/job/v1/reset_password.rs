use crate::queue::{Job, JobError, SendInvitationEmail, SharedJobState, V1};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ResetPassword {
    pub membership_id: Uuid,
    pub user_id: String,
}

impl ResetPassword {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        _db: &impl ConnectionTrait,
    ) -> Result<Option<Job>, JobError> {
        let action_url = job_state.auth0_client.password_reset(&self.user_id).await?;
        Ok(Some(Job::from(SendInvitationEmail {
            membership_id: self.membership_id,
            action_url,
            message_id: Uuid::new_v4(),
        })))
    }
}

impl From<ResetPassword> for Job {
    fn from(value: ResetPassword) -> Self {
        Self::V1(V1::ResetPassword(value))
    }
}

impl PartialEq<Job> for ResetPassword {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::ResetPassword(j)) if j == self)
    }
}
impl PartialEq<ResetPassword> for Job {
    fn eq(&self, other: &ResetPassword) -> bool {
        matches!(self, Job::V1(V1::ResetPassword(j)) if j == other)
    }
}
