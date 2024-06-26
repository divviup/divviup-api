use super::*;
use crate::{
    clients::aggregator_client::api_types::{AggregatorVdaf, AuthenticationToken, QueryType},
    entity::{Account, CollectorCredential, Protocol, Task},
    handler::Error,
    Crypter,
};
use sea_orm::IntoActiveModel;
use std::fmt::Debug;
use trillium_client::Client;

#[derive(Clone, Debug)]
pub struct ProvisionableTask {
    pub account: Account,
    pub id: String,
    pub vdaf_verify_key: String,
    pub name: String,
    pub leader_aggregator: Aggregator,
    pub helper_aggregator: Aggregator,
    pub vdaf: Vdaf,
    pub aggregator_vdaf: AggregatorVdaf,
    pub min_batch_size: u64,
    pub max_batch_size: Option<u64>,
    pub batch_time_window_size_seconds: Option<u64>,
    pub expiration: Option<OffsetDateTime>,
    pub time_precision_seconds: u64,
    pub collector_credential: CollectorCredential,
    pub aggregator_auth_token: Option<String>,
    pub protocol: Protocol,
}

impl ProvisionableTask {
    async fn provision_aggregator(
        &self,
        http_client: Client,
        aggregator: Aggregator,
        crypter: &Crypter,
    ) -> Result<TaskResponse, Error> {
        let response = aggregator
            .client(http_client, crypter)?
            .create_task(self)
            .await?;

        assert_same(&self.aggregator_vdaf, &response.vdaf, "vdaf")?;
        assert_same(
            self.min_batch_size,
            response.min_batch_size,
            "min_batch_size",
        )?;
        assert_same(&self.query_type(), &response.query_type, "query_type")?;
        assert_same(
            // precision is lost in the round trip so we truncate our own
            self.expiration
                .map(|t| t.replace_millisecond(0))
                .transpose()?,
            response.task_expiration()?,
            "task_expiration",
        )?;
        assert_same(
            self.time_precision_seconds,
            response.time_precision.as_seconds(),
            "time_precision",
        )?;

        assert_same(&*self.id, &*response.task_id.to_string(), "task_id")?;

        // there are likely some more validations needed
        Ok(response)
    }

    pub async fn provision(
        mut self,
        client: Client,
        crypter: &Crypter,
    ) -> Result<ActiveModel, Error> {
        let helper = self
            .provision_aggregator(client.clone(), self.helper_aggregator.clone(), crypter)
            .await?;

        self.aggregator_auth_token = helper.aggregator_auth_token.map(AuthenticationToken::token);

        let _leader = self
            .provision_aggregator(client, self.leader_aggregator.clone(), crypter)
            .await?;

        Ok(Task {
            id: self.id,
            account_id: self.account.id,
            name: self.name,
            vdaf: self.vdaf.into(),
            min_batch_size: self.min_batch_size.try_into()?,
            max_batch_size: self.max_batch_size.map(TryInto::try_into).transpose()?,
            batch_time_window_size_seconds: self
                .batch_time_window_size_seconds
                .map(TryInto::try_into)
                .transpose()?,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
            time_precision_seconds: self.time_precision_seconds.try_into()?,
            report_count: 0,
            aggregate_collection_count: 0,
            expiration: self.expiration,
            leader_aggregator_id: self.leader_aggregator.id,
            helper_aggregator_id: self.helper_aggregator.id,
            collector_credential_id: self.collector_credential.id,
            report_counter_interval_collected: 0,
            report_counter_decode_failure: 0,
            report_counter_decrypt_failure: 0,
            report_counter_expired: 0,
            report_counter_outdated_key: 0,
            report_counter_success: 0,
            report_counter_too_early: 0,
            report_counter_task_expired: 0,
        }
        .into_active_model())
    }

    pub fn query_type(&self) -> QueryType {
        if let Some(max_batch_size) = self.max_batch_size {
            QueryType::FixedSize {
                max_batch_size,
                batch_time_window_size: self.batch_time_window_size_seconds,
            }
        } else {
            QueryType::TimeInterval
        }
    }
}
