use crate::{
    clients::{Auth0Client, ClientError, PostmarkClient},
    entity::Membership,
    Config,
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use trillium::{Method, Status};
use trillium_client::Client;
use url::Url;

mod v1;
pub use v1::{
    CreateUser, QueueCleanup, ResetPassword, SendInvitationEmail, SessionCleanup, TaskSync, V1,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "version")]
pub enum Job {
    V1(V1),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Error)]
pub enum JobError {
    #[error("{0}")]
    Db(String),

    #[error("{0} with id {1} was not found")]
    MissingRecord(String, String),

    #[error("{0}")]
    ClientOther(String),

    #[error("unexpected http status {method} {url} {status:?}: {body}")]
    HttpStatusNotSuccess {
        method: Method,
        url: Url,
        status: Option<Status>,
        body: String,
    },
}

impl JobError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Db(_) | Self::ClientOther(_) | Self::HttpStatusNotSuccess { .. }
        )
    }
}

impl From<DbErr> for JobError {
    fn from(value: DbErr) -> Self {
        Self::Db(value.to_string())
    }
}

impl From<ClientError> for JobError {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::HttpStatusNotSuccess {
                method,
                url,
                status,
                body,
            } => Self::HttpStatusNotSuccess {
                method,
                url,
                status,
                body,
            },
            other => Self::ClientOther(other.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct SharedJobState {
    pub auth0_client: Auth0Client,
    pub postmark_client: PostmarkClient,
    pub http_client: Client,
    pub crypter: crate::Crypter,
}
impl From<&Config> for SharedJobState {
    fn from(config: &Config) -> Self {
        Self {
            auth0_client: Auth0Client::new(config),
            postmark_client: PostmarkClient::new(config),
            http_client: config.client.clone(),
            crypter: config.crypter.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnqueueJob {
    pub job: Job,
    pub scheduled: Option<OffsetDateTime>,
}

impl EnqueueJob {
    pub fn scheduled_at(mut self, scheduled: OffsetDateTime) -> Self {
        self.scheduled = Some(scheduled);
        self
    }

    pub fn scheduled_in(self, duration: Duration) -> Self {
        self.scheduled_at(OffsetDateTime::now_utc() + duration)
    }
}

impl<T: Into<Job>> From<T> for EnqueueJob {
    fn from(value: T) -> Self {
        EnqueueJob {
            job: value.into(),
            scheduled: None,
        }
    }
}

impl Job {
    pub fn new_invitation_flow(membership: &Membership) -> Self {
        Self::from(CreateUser {
            membership_id: membership.id,
        })
    }

    pub async fn perform(
        &mut self,
        job_state: &SharedJobState,
        db: &impl ConnectionTrait,
    ) -> Result<Option<EnqueueJob>, JobError> {
        match self {
            Job::V1(job) => job.perform(job_state, db).await,
        }
    }

    pub async fn insert(
        self,
        db: &impl ConnectionTrait,
    ) -> Result<crate::entity::queue::Model, DbErr> {
        crate::entity::queue::ActiveModel::from(self)
            .insert(db)
            .await
    }
}
