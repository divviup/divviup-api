mod create_user;
mod reset_password;
mod send_invitation_email;

use super::{Job, JobError, SharedJobState};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

pub use create_user::CreateUser;
pub use reset_password::ResetPassword;
pub use send_invitation_email::SendInvitationEmail;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum V1 {
    SendInvitationEmail(SendInvitationEmail),
    CreateUser(CreateUser),
    ResetPassword(ResetPassword),
}

impl V1 {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<Job>, JobError> {
        match self {
            V1::SendInvitationEmail(job) => job.perform(job_state, db).await,
            V1::CreateUser(job) => job.perform(job_state, db).await,
            V1::ResetPassword(job) => job.perform(job_state, db).await,
        }
    }
}
