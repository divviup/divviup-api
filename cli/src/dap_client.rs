use crate::{CliResult, DetermineAccountId, Error};
use clap::Subcommand;
use divviup_client::{self, Histogram, Protocol};
use janus_messages::Duration;
use prio::vdaf::prio3::{Prio3Count, Prio3Histogram, Prio3Sum, Prio3SumVec};
use tokio::try_join;

#[derive(Subcommand, Debug)]
pub enum DapClientAction {
    /// upload a report
    Upload {
        /// DAP task to upload to, as an unpadded Base64, URL-safe encoded string.
        #[arg(long)]
        task_id: String,
        /// The measurement to upload.
        #[arg(long)]
        measurement: String,
    },
}

impl DapClientAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: divviup_client::DivviupClient,
    ) -> CliResult {
        let account_id = account_id.await?;

        match self {
            DapClientAction::Upload {
                task_id,
                measurement,
            } => {
                let (task, aggregators) =
                    try_join!(client.task(&task_id), client.aggregators(account_id))?;

                if !aggregators.iter().all(|a| a.protocol == Protocol::Dap09) {
                    return Err(Error::Other("unable to handle protocol version".into()));
                }

                let leader_url = aggregators
                    .iter()
                    .find(|a| a.id == task.leader_aggregator_id)
                    .map(|a| a.dap_url.clone())
                    .ok_or_else(|| Error::Other("leader URL unset".into()))?;

                let helper_url = aggregators
                    .iter()
                    .find(|a| a.id == task.helper_aggregator_id)
                    .map(|a| a.dap_url.clone())
                    .ok_or_else(|| Error::Other("helper URL unset".into()))?;

                match task.vdaf {
                    divviup_client::Vdaf::Count => {
                        let client = janus_client::Client::new(
                            task_id.parse()?,
                            leader_url,
                            helper_url,
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Count::new_count(2)?,
                        )
                        .await?;
                        let v = measurement.parse().map_err(|e| {
                            Error::Other(format!("cannot parse measurement as boolean: {e:?}"))
                        })?;
                        client.upload(&v).await?;
                    }
                    divviup_client::Vdaf::Sum { bits } => {
                        let client = janus_client::Client::new(
                            task_id.parse()?,
                            leader_url,
                            helper_url,
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Sum::new_sum(2, bits as usize)?,
                        )
                        .await?;
                        let v = measurement.parse().map_err(|e| {
                            Error::Other(format!("cannot parse measurement as number: {e:?}"))
                        })?;
                        client.upload(&v).await?;
                    }
                    divviup_client::Vdaf::SumVec {
                        bits,
                        length,
                        chunk_length,
                    } => {
                        // Chunk length should always be set for DAP-09 tasks
                        let chunk_length = chunk_length.ok_or_else(|| {
                            Error::Other(
                                "task's VDAF configuration does not provide a chunk length".into(),
                            )
                        })?;
                        let client = janus_client::Client::new(
                            task_id.parse()?,
                            leader_url,
                            helper_url,
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3SumVec::new_sum_vec(
                                2,
                                bits as usize,
                                length as usize,
                                chunk_length as usize,
                            )?,
                        )
                        .await?;
                        let v = measurement
                            .split(',')
                            .map(|s| s.trim().parse())
                            .collect::<Result<_, _>>()
                            .map_err(|e| {
                                Error::Other(format!("cannot parse measurement as number: {e:?}"))
                            })?;
                        client.upload(&v).await?;
                    }
                    divviup_client::Vdaf::Histogram(Histogram::Categorical {
                        buckets,
                        chunk_length,
                    }) => {
                        // Chunk length should always be set for DAP-09 tasks
                        let chunk_length = chunk_length.ok_or_else(|| {
                            Error::Other(
                                "task's VDAF configuration does not provide a chunk length".into(),
                            )
                        })?;
                        let client = janus_client::Client::new(
                            task_id.parse()?,
                            leader_url,
                            helper_url,
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Histogram::new_histogram(2, buckets.len(), chunk_length as usize)?,
                        )
                        .await?;
                        let v = measurement.parse().map_err(|e| {
                            Error::Other(format!("cannot parse measurement as number: {e:?}"))
                        })?;
                        client.upload(&v).await?;
                    }
                    divviup_client::Vdaf::Histogram(Histogram::Continuous {
                        buckets,
                        chunk_length,
                    }) => {
                        // Chunk length should always be set for DAP-09 tasks
                        let chunk_length = chunk_length.ok_or_else(|| {
                            Error::Other(
                                "task's VDAF configuration does not provide a chunk length".into(),
                            )
                        })?;
                        let client = janus_client::Client::new(
                            task_id.parse()?,
                            leader_url,
                            helper_url,
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Histogram::new_histogram(2, buckets.len(), chunk_length as usize)?,
                        )
                        .await?;
                        let v = measurement.parse().map_err(|e| {
                            Error::Other(format!("cannot parse measurement as number: {e:?}"))
                        })?;
                        client.upload(&v).await?;
                    }
                    _ => {
                        return Err(Error::Other("No matching VDAF".into()));
                    }
                };
            }
        }

        Ok(())
    }
}
