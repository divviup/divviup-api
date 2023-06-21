use super::{ActiveModel, *};
use crate::{
    entity::{Account, Aggregator},
    handler::Error,
};
use janus_messages::HpkeConfig;
use trillium_client::Client;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum TaskProvisioningError {
    #[error("discrepancy in {0}")]
    Discrepancy(&'static str),
    #[error("missing task id from all sources")]
    MissingTaskId,
}

#[derive(Clone, Debug)]
pub struct ProvisionableTask {
    pub account: Account,
    pub id: Option<String>,
    pub vdaf_verify_key: String,
    pub aggregator_auth_token: Option<String>,
    pub collector_auth_token: Option<String>,
    pub name: String,
    pub leader_aggregator: Aggregator,
    pub helper_aggregator: Aggregator,
    pub vdaf: Vdaf,
    pub min_batch_size: u64,
    pub max_batch_size: Option<u64>,
    pub expiration: Option<OffsetDateTime>,
    pub time_precision_seconds: u64,
    pub hpke_config: HpkeConfig,
}

fn assert_same<T: Eq>(
    ours: T,
    theirs: T,
    property: &'static str,
) -> Result<(), TaskProvisioningError> {
    if ours == theirs {
        Ok(())
    } else {
        Err(TaskProvisioningError::Discrepancy(property))
    }
}

impl ProvisionableTask {
    async fn provision_aggregator(
        &mut self,
        http_client: Client,
        aggregator: Aggregator,
    ) -> Result<Option<TaskResponse>, Error> {
        let Some(client) = aggregator.client(http_client) else { return Ok(None) };
        let response = client.create_task(self).await?;

        assert_same(&self.vdaf, &response.vdaf.clone().into(), "vdaf")?;
        assert_same(
            self.min_batch_size,
            response.min_batch_size,
            "min_batch_size",
        )?;
        assert_same(
            &self.max_batch_size.into(),
            &response.query_type,
            "query_type",
        )?;
        assert_same(
            self.expiration,
            response.task_expiration()?,
            "task_expiration",
        )?;
        assert_same(
            self.time_precision_seconds,
            response.time_precision.as_seconds(),
            "time_precision_seconds",
        )?;

        if let Some(id) = self.id.as_deref() {
            assert_same(id, &*response.task_id.to_string(), "task_id")?;
        } else {
            self.id = Some(response.task_id.to_string());
        }

        // there are likely some more validations needed
        Ok(Some(response))
    }

    pub async fn provision(mut self, client: Client) -> Result<ActiveModel, Error> {
        let _leader = self
            .provision_aggregator(client.clone(), self.leader_aggregator.clone())
            .await?;
        let _helper = self
            .provision_aggregator(client, self.helper_aggregator.clone())
            .await?;

        Ok(super::Model {
            id: self.id.ok_or(TaskProvisioningError::MissingTaskId)?,
            account_id: self.account.id,
            name: self.name,
            vdaf: self.vdaf,
            min_batch_size: self.min_batch_size.try_into()?,
            max_batch_size: self.max_batch_size.map(TryInto::try_into).transpose()?,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            time_precision_seconds: self.time_precision_seconds.try_into()?,
            report_count: 0,
            aggregate_collection_count: 0,
            expiration: self.expiration,
            leader_aggregator_id: self.leader_aggregator.id,
            helper_aggregator_id: self.helper_aggregator.id,
        }
        .into_active_model())
    }
}
