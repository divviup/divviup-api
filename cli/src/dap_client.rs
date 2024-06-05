use crate::{CliResult, DetermineAccountId, Error};
use anyhow::Context;
use clap::{ArgAction, Args, Subcommand};
use divviup_client::{self, Protocol};
use janus_collector::{Collector, PrivateCollectorCredential};
use janus_messages::{BatchId, Duration, FixedSizeQuery, Interval, Query, Time};
use prio::vdaf::prio3::{Prio3Count, Prio3Histogram, Prio3Sum, Prio3SumVec};
use std::{fs::File, io::BufReader, path::PathBuf};
use tokio::try_join;

macro_rules! query_dispatch {
    ($query:expr, ($janus_query:ident) => $body:tt) => {
        match (
            &$query.batch_interval_start,
            &$query.batch_interval_duration,
            &$query.batch_id,
            $query.current_batch,
        ) {
            (Some(batch_interval_start), Some(batch_interval_duration), None, false) => {
                let $janus_query = Query::new_time_interval(
                    Interval::new(
                        Time::from_seconds_since_epoch(*batch_interval_start),
                        Duration::from_seconds(*batch_interval_duration),
                    )
                    .context("illegal time interval")?,
                );
                $body
            }
            (None, None, Some(batch_id), false) => {
                let $janus_query = Query::new_fixed_size(FixedSizeQuery::ByBatchId {
                    batch_id: *batch_id,
                });
                $body
            }
            (None, None, None, true) => {
                let $janus_query = Query::new_fixed_size(FixedSizeQuery::CurrentBatch);
                $body
            }
            _ => unreachable!("clap argument parsing shouldn't allow this to be possible"),
        }
    };
}

macro_rules! vdaf_dispatch {
    ($divviup_vdaf:expr, ($janus_vdaf:ident) => $body:tt) => {
        match $divviup_vdaf {
            divviup_client::Vdaf::Count => {
                let $janus_vdaf = Prio3Count::new_count(2).context("failed to instantiate VDAF")?;
                $body
            }
            divviup_client::Vdaf::Sum { bits } => {
                let $janus_vdaf =
                    Prio3Sum::new_sum(2, bits as usize).context("failed to instantiate VDAF")?;
                $body
            }
            divviup_client::Vdaf::SumVec(sum_vec) => {
                let $janus_vdaf = Prio3SumVec::new_sum_vec(
                    2,
                    sum_vec.bits as usize,
                    sum_vec.length as usize,
                    sum_vec.chunk_length(),
                )
                .context("failed to instantiate VDAF")?;
                $body
            }
            divviup_client::Vdaf::Histogram(histogram) => {
                let $janus_vdaf =
                    Prio3Histogram::new_histogram(2, histogram.length(), histogram.chunk_length())
                        .context("failed to instantiate VDAF")?;
                $body
            }
            v => {
                return Err(Error::Other(format!("No matching VDAF {v:?}")));
            }
        }
    };
}

macro_rules! collect_dispatch {
    ($vdaf:expr, $divviup_query:expr, ($janus_query:ident, $janus_vdaf:ident) => $body:tt) => {
        query_dispatch!($divviup_query, ($janus_query) => {
            vdaf_dispatch!($vdaf, ($janus_vdaf) => {
                let body = $body;
                body
            })
        })
    }
}

#[derive(Debug, Args, PartialEq, Eq)]
#[group(required = true)]
pub struct QueryOptions {
    /// Start of the collection batch interval, as the number of seconds since the Unix epoch
    #[clap(
        long,
        requires = "batch_interval_duration",
        help_heading = "Collect Request Parameters (Time Interval)"
    )]
    batch_interval_start: Option<u64>,
    /// Duration of the collection batch interval, in seconds
    #[clap(
        long,
        requires = "batch_interval_start",
        help_heading = "Collect Request Parameters (Time Interval)"
    )]
    batch_interval_duration: Option<u64>,

    /// Batch identifier, encoded with base64url
    #[clap(
        long,
        conflicts_with_all = ["batch_interval_start", "batch_interval_duration", "current_batch"],
        help_heading = "Collect Request Parameters (Fixed Size)",
    )]
    batch_id: Option<BatchId>,
    /// Have the aggregator select a batch that has not yet been collected
    #[clap(
        long,
        action = ArgAction::SetTrue,
        conflicts_with_all = ["batch_interval_start", "batch_interval_duration", "batch_id"],
        help_heading = "Collect Request Parameters (Fixed Size)",
    )]
    current_batch: bool,
}

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
    /// collect an aggregate result
    Collect {
        /// DAP task to collect from, as an unpadded Base64, URL-safe encoded string.
        #[arg(long)]
        task_id: String,

        /// Path to a file containing private collector credentials.
        ///
        /// This can be obtained with the command `divviup collector-credential generate`.
        #[clap(long, default_value = "./collector-credential.json")]
        collector_credential_file: PathBuf,

        #[clap(flatten)]
        query: QueryOptions,
    },
}

impl DapClientAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: divviup_client::DivviupClient,
    ) -> CliResult {
        let account_id = account_id.await?;

        let task_id = match self {
            DapClientAction::Upload { ref task_id, .. } => task_id,
            DapClientAction::Collect { ref task_id, .. } => task_id,
        };

        let (task, aggregators) = try_join!(client.task(task_id), client.aggregators(account_id))?;
        let task_id = task_id.parse().context("failed to parse task ID")?;
        let time_precision = Duration::from_seconds(task.time_precision_seconds as u64);

        let (leader_url, leader_protocol) = aggregators
            .iter()
            .find(|a| a.id == task.leader_aggregator_id)
            .map(|a| (a.dap_url.clone(), a.protocol))
            .ok_or_else(|| Error::Other("leader URL unset".into()))?;

        let (helper_url, helper_protocol) = aggregators
            .iter()
            .find(|a| a.id == task.helper_aggregator_id)
            .map(|a| (a.dap_url.clone(), a.protocol))
            .ok_or_else(|| Error::Other("helper URL unset".into()))?;

        if ![leader_protocol, helper_protocol]
            .iter()
            .all(|p| p == &Protocol::Dap09)
        {
            return Err(Error::Other("unable to handle protocol version".into()));
        }

        match self {
            DapClientAction::Upload { measurement, .. } => {
                vdaf_dispatch!(task.vdaf, (janus_vdaf) => {
                    let v = janus_vdaf.parse_measurement(measurement).context("failed to parse measurement")?;
                    let client = janus_client::Client::new(
                        task_id,
                        leader_url,
                        helper_url,
                        time_precision,
                        janus_vdaf,
                    )
                    .await
                    .context("failed to instantiate client")?;
                    Ok(client.upload(&v).await.context("failed to upload")?)
                })
            }
            DapClientAction::Collect {
                collector_credential_file,
                query,
                ..
            } => {
                let credential: PrivateCollectorCredential =
                    serde_json::from_reader(BufReader::new(File::open(collector_credential_file)?))
                        .context("failed to load collector credential")?;
                collect_dispatch!(task.vdaf, query, (janus_query, janus_vdaf) => {
                    let collector = Collector::new(
                        task_id,
                        leader_url,
                        credential.authentication_token(),
                        credential.hpke_keypair(),
                        janus_vdaf
                    ).context("failed to instantiate collector")?;
                    let collection = collector.collect(
                        janus_query,
                        // For now, we only support Prio VDAFs and thus always use () as the
                        // aggregation parameter
                        &()
                    ).await.context("failed to run collection job")?;

                    let (start, duration) = collection.interval();
                    println!("Number of reports: {}", collection.report_count());
                    println!("Interval start: {}", start);
                    println!("Interval end: {}", *start + *duration);
                    println!(
                        "Interval length: {:?}",
                        // `std::time::Duration` has the most human-readable debug print for a Duration.
                        duration.to_std().map_err(|err| Error::Anyhow(err.into()))?
                    );
                    println!("Aggregation result: {:?}", collection.aggregate_result());
                    println!("collection: {collection:?}");

                    Ok(())
                })
            }
        }
    }
}

trait ParseMeasurement: prio::vdaf::Vdaf {
    fn parse_measurement<I: AsRef<str>>(&self, measurement: I) -> CliResult<Self::Measurement>;
}

impl ParseMeasurement for Prio3Count {
    fn parse_measurement<I: AsRef<str>>(&self, measurement: I) -> CliResult<Self::Measurement> {
        Ok(measurement
            .as_ref()
            .parse()
            .context("failed to parse measurement")?)
    }
}

impl ParseMeasurement for Prio3Sum {
    fn parse_measurement<I: AsRef<str>>(&self, measurement: I) -> CliResult<Self::Measurement> {
        Ok(measurement
            .as_ref()
            .parse()
            .context("failed to parse measurement")?)
    }
}

impl ParseMeasurement for Prio3SumVec {
    fn parse_measurement<I: AsRef<str>>(&self, measurement: I) -> CliResult<Self::Measurement> {
        Ok(measurement
            .as_ref()
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<_, _>>()
            .context("failed to parse measurement")?)
    }
}

impl ParseMeasurement for Prio3Histogram {
    fn parse_measurement<I: AsRef<str>>(&self, measurement: I) -> CliResult<Self::Measurement> {
        Ok(measurement
            .as_ref()
            .parse()
            .context("failed to parse measurement")?)
    }
}
