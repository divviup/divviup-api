use std::str::FromStr;
use url::Url;
use janus_client;
use janus_messages::{Duration, TaskId};
use prio::vdaf::prio3::{Prio3Histogram, Prio3SumVec, Prio3Sum, Prio3Count};
use crate::{CliResult, DetermineAccountId, Error};
use clap::Subcommand;
use divviup_client::{self, Histogram, Protocol};

#[derive(Subcommand, Debug)]
pub enum DapClientAction {
    /// create a new task for the target account
    Upload {
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        value: String,
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
            DapClientAction::Upload { task_id, value } => {
                let task = client.task(&task_id).await?;

                let aggregators = client.aggregators(account_id).await?;

                if !aggregators.iter().all(|a| a.protocol == Protocol::Dap09) {
                    return Err(Error::Other("unable to handle protocol version".into()));
                }

                let mut leader_url: Option<Url> = None;
                let mut helper_url: Option<Url> = None;

                for a in &aggregators {
                    if a.id == task.leader_aggregator_id {
                        leader_url = Some(a.dap_url.clone());
                    }
                    if a.id == task.helper_aggregator_id {
                        helper_url = Some(a.dap_url.clone());
                    }
                }
                if leader_url.is_none() {
                    return Err(Error::Other("leader URL unset".into()));
                }
                if helper_url.is_none() {
                    return Err(Error::Other("leader URL unset".into()));
                }

                match task.vdaf {
                    divviup_client::Vdaf::Count => {
                        let client = janus_client::Client::new(
                            TaskId::from_str(&task_id).unwrap(),
                            leader_url.unwrap(),
                            helper_url.unwrap(),
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Count::new_count(2).unwrap(),
                        )
                        .await
                        .unwrap();
                        let v = bool::from_str(&value).unwrap();
                        client.upload(&v).await.unwrap();
                        println!("uploaded measurement: {}", &value);
                    }
                    divviup_client::Vdaf::Sum{bits} => {
                        let client = janus_client::Client::new(
                            TaskId::from_str(&task_id).unwrap(),
                            leader_url.unwrap(),
                            helper_url.unwrap(),
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Sum::new_sum(2, bits as usize).unwrap(),
                        )
                        .await
                        .unwrap();
                        let v = u128::from_str(&value).unwrap();
                        client.upload(&v).await.unwrap();
                        println!("uploaded measurement: {}", &value);
                    }
                    divviup_client::Vdaf::SumVec { bits, length, chunk_length } => {
                        let client = janus_client::Client::new(
                            TaskId::from_str(&task_id).unwrap(),
                            leader_url.unwrap(),
                            helper_url.unwrap(),
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3SumVec::new_sum_vec(2, bits as usize, length as usize, chunk_length.unwrap() as usize).unwrap(),
                        )
                        .await
                        .unwrap();
                        let v: Vec<u128> = value
                            .split(',')
                            .map(|s| u128::from_str(s.trim()).unwrap())
                            .collect();
                        client.upload(&v).await.unwrap();
                        println!("uploaded measurement: {}", &value);
                    }
                    divviup_client::Vdaf::Histogram(Histogram::Categorical {
                        buckets,
                        chunk_length,
                    }) => {
                        let client = janus_client::Client::new(
                            TaskId::from_str(&task_id).unwrap(),
                            leader_url.unwrap(),
                            helper_url.unwrap(),
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Histogram::new_histogram(
                                2,
                                buckets.len(),
                                chunk_length.unwrap() as usize,
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                        let v = usize::from_str(&value).unwrap();
                        client.upload(&v).await.unwrap();
                        println!("uploaded measurement: {}", &value);
                    }
                    divviup_client::Vdaf::Histogram(Histogram::Continuous {
                        buckets,
                        chunk_length,
                    }) => {
                        let client = janus_client::Client::new(
                            TaskId::from_str(&task_id).unwrap(),
                            leader_url.unwrap(),
                            helper_url.unwrap(),
                            Duration::from_seconds(task.time_precision_seconds as u64),
                            Prio3Histogram::new_histogram(
                                2,
                                buckets.len(),
                                chunk_length.unwrap() as usize,
                            )
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                        let v = usize::from_str(&value).unwrap();
                        client.upload(&v).await.unwrap();
                        println!("uploaded measurement: {}", &value);
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
