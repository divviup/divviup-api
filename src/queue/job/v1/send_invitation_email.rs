use crate::{
    entity::*,
    queue::{EnqueueJob, Job, JobError, SharedJobState, V1},
};
use sea_orm::{ConnectionTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SendInvitationEmail {
    pub membership_id: Uuid,
    pub action_url: Url,
    pub message_id: Uuid,
}

impl SendInvitationEmail {
    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        let (membership, account) = Memberships::find_by_id(self.membership_id)
            .find_also_related(Accounts)
            .one(db)
            .await?
            .ok_or_else(|| {
                JobError::MissingRecord(String::from("membership"), self.membership_id.to_string())
            })?;

        let account = account.ok_or_else(|| {
            JobError::MissingRecord(String::from("account"), membership.account_id.to_string())
        })?;

        job_state
            .postmark_client
            .send_email_template(
                &membership.user_email,
                "user-invitation",
                &json!({
                    "email": membership.user_email,
                    "account_name": &account.name,
                    "action_url": self.action_url
                }),
                Some(self.message_id.to_string()),
            )
            .await?;

        Ok(None)
    }
}

impl From<SendInvitationEmail> for Job {
    fn from(value: SendInvitationEmail) -> Self {
        Self::V1(V1::SendInvitationEmail(value))
    }
}
impl PartialEq<Job> for SendInvitationEmail {
    fn eq(&self, other: &Job) -> bool {
        matches!(other, Job::V1(V1::SendInvitationEmail(j)) if j == self)
    }
}

impl PartialEq<SendInvitationEmail> for Job {
    fn eq(&self, other: &SendInvitationEmail) -> bool {
        matches!(self, Job::V1(V1::SendInvitationEmail(j)) if j == other)
    }
}
