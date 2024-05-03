use std::str::FromStr;
use url::Url;

use janus_client;

use janus_messages::{Duration, TaskId};
use prio::vdaf::prio3::Prio3Histogram;

use crate::{CliResult, DetermineAccountId, Error, Output};
use clap::Subcommand;
use divviup_client::{self, Histogram};

#[derive(Subcommand, Debug)]
pub enum DapClientAction {
    /// create a new task for the target account
    Upload {
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        value: usize,
    },
}

impl DapClientAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: divviup_client::DivviupClient,
        _output: Output,
    ) -> CliResult {
        let account_id = account_id.await?;

        match self {
            DapClientAction::Upload { task_id, value } => {
                let task = client.task(&task_id).await?;

                let aggregators = client.aggregators(account_id).await?;

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
                        client.upload(&value).await.unwrap();
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
