mod create_user;
mod queue_cleanup;
mod reset_password;
mod send_invitation_email;
mod session_cleanup;
mod task_sync;

use crate::queue::EnqueueJob;

use super::{JobError, SharedJobState};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

pub use create_user::CreateUser;
pub use queue_cleanup::QueueCleanup;
pub use reset_password::ResetPassword;
pub use send_invitation_email::SendInvitationEmail;
pub use session_cleanup::SessionCleanup;
pub use task_sync::TaskSync;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum V1 {
    SendInvitationEmail(SendInvitationEmail),
    CreateUser(CreateUser),
    ResetPassword(ResetPassword),
    SessionCleanup(SessionCleanup),
    QueueCleanup(QueueCleanup),
    TaskSync(TaskSync),
}

impl V1 {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        match self {
            V1::SendInvitationEmail(job) => job.perform(job_state, db).await,
            V1::CreateUser(job) => job.perform(job_state, db).await,
            V1::ResetPassword(job) => job.perform(job_state, db).await,
            V1::SessionCleanup(job) => job.perform(job_state, db).await,
            V1::QueueCleanup(job) => job.perform(job_state, db).await,
            V1::TaskSync(job) => job.perform(job_state, db).await,
        }
    }
}
