use crate::{CliResult, DetermineAccountId, Error, Output};
use clap::Subcommand;
use divviup_client::{
    dp_strategy::{self, PureDpBudget, PureDpDiscreteLaplace},
    BigUint, DivviupClient, Histogram, NewTask, Ratio, SumVec, Uuid, Vdaf,
};
use humantime::{Duration, Timestamp};
use std::time::SystemTime;
use time::OffsetDateTime;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum VdafName {
    Count,
    Histogram,
    Sum,
    CountVec,
    SumVec,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DpStrategy {
    PureDpDiscreteLaplace,
}

#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// list all tasks for the target account
    List,

    /// retrieve details of a single task. this also refreshes cached data, such as metrics.
    Get { task_uuid: String },

    /// create a new task for the target account
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        leader_aggregator_id: Uuid,
        #[arg(long)]
        helper_aggregator_id: Uuid,
        #[arg(long)]
        vdaf: VdafName,
        #[arg(long)]
        min_batch_size: u64,
        #[arg(long)]
        max_batch_size: Option<u64>,
        #[arg(long, requires = "max_batch_size")]
        batch_time_window_size: Option<Duration>,
        #[arg(long)]
        time_precision: Duration,
        #[arg(long)]
        collector_credential_id: Uuid,
        #[arg(long, value_delimiter = ',')]
        categorical_buckets: Option<Vec<String>>,
        #[arg(long, value_delimiter = ',')]
        continuous_buckets: Option<Vec<u64>>,
        #[arg(long, required_if_eq_any([("vdaf", "count_vec"), ("vdaf", "sum_vec")]))]
        length: Option<u64>,
        #[arg(long, required_if_eq_any([("vdaf", "sum"), ("vdaf", "sum_vec")]))]
        bits: Option<u8>,
        #[arg(long)]
        chunk_length: Option<u64>,
        #[arg(long, requires = "differential_privacy_epsilon")]
        differential_privacy_strategy: Option<DpStrategy>,
        #[arg(long, requires = "differential_privacy_strategy")]
        differential_privacy_epsilon: Option<f64>,
    },

    /// rename a task
    Rename { task_uuid: String, name: String },

    /// delete a task
    Delete {
        task_uuid: String,
        /// delete the task even if the aggregators are unreachable
        #[arg(long, action)]
        force: bool,
    },

    /// set the expiration date of a task
    SetExpiration {
        task_uuid: String,
        /// the date and time to set to.
        ///
        /// if omitted, unset the expiration. the format is RFC 3339.
        expiration: Option<Timestamp>,
        /// set the expiration to the current time, effectively disabling the task.
        #[arg(long, action, conflicts_with = "expiration")]
        now: bool,
    },

    /// retrieve the collector auth tokens for a task
    CollectorAuthTokens { task_uuid: String },
}

impl TaskAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        let account_id = account_id.await?;

        match self {
            TaskAction::List => output.display(client.tasks(account_id).await?),
            TaskAction::Get { task_uuid } => output.display(client.task(&task_uuid).await?),
            TaskAction::Create {
                name,
                leader_aggregator_id,
                helper_aggregator_id,
                vdaf,
                min_batch_size,
                max_batch_size,
                batch_time_window_size,
                time_precision,
                collector_credential_id,
                categorical_buckets,
                continuous_buckets,
                length,
                bits,
                chunk_length,
                differential_privacy_strategy,
                differential_privacy_epsilon,
            } => {
                let vdaf = match vdaf {
                    VdafName::Count => {
                        if differential_privacy_strategy.is_some()
                            || differential_privacy_epsilon.is_some()
                        {
                            return Err(Error::Other(
                                "differential privacy noise is not yet supported with Prio3Count"
                                    .into(),
                            ));
                        }
                        Vdaf::Count
                    }
                    VdafName::Histogram => {
                        let dp_strategy =
                            match (differential_privacy_strategy, differential_privacy_epsilon) {
                                (None, None) => dp_strategy::Prio3Histogram::NoDifferentialPrivacy,
                                (None, Some(_)) => {
                                    return Err(Error::Other(
                                        "missing differential-privacy-strategy".into(),
                                    ))
                                }
                                (Some(_), None) => {
                                    return Err(Error::Other(
                                        "missing differential-privacy-epsilon".into(),
                                    ))
                                }
                                (Some(DpStrategy::PureDpDiscreteLaplace), Some(epsilon)) => {
                                    dp_strategy::Prio3Histogram::PureDpDiscreteLaplace(
                                        PureDpDiscreteLaplace {
                                            budget: PureDpBudget {
                                                epsilon: float_to_biguint_ratio(epsilon)
                                                    .ok_or_else(|| {
                                                        Error::Other("invalid epsilon".into())
                                                    })?,
                                            },
                                        },
                                    )
                                }
                            };
                        match (length, categorical_buckets, continuous_buckets) {
                            (Some(length), None, None) => Vdaf::Histogram(Histogram::Length {
                                length,
                                chunk_length,
                                dp_strategy,
                            }),
                            (None, Some(buckets), None) => {
                                Vdaf::Histogram(Histogram::Categorical {
                                    buckets,
                                    chunk_length,
                                    dp_strategy,
                                })
                            }
                            (None, None, Some(buckets)) => Vdaf::Histogram(Histogram::Continuous {
                                buckets,
                                chunk_length,
                                dp_strategy,
                            }),
                            (None, None, None) => {
                                return Err(Error::Other("continuous-buckets, categorical-buckets, or length are required for histogram vdaf".into()));
                            }
                            _ => {
                                return Err(Error::Other("continuous-buckets, categorical-buckets, and length are mutually exclusive".into()));
                            }
                        }
                    }
                    VdafName::Sum => {
                        if differential_privacy_strategy.is_some()
                            || differential_privacy_epsilon.is_some()
                        {
                            return Err(Error::Other(
                                "differential privacy noise is not yet supported with Prio3Sum"
                                    .into(),
                            ));
                        }
                        Vdaf::Sum {
                            bits: bits.unwrap(),
                        }
                    }
                    VdafName::CountVec => {
                        if differential_privacy_strategy.is_some()
                            || differential_privacy_epsilon.is_some()
                        {
                            return Err(Error::Other(
                                "differential privacy noise is not supported with Prio3CountVec"
                                    .into(),
                            ));
                        }
                        Vdaf::CountVec {
                            length: length.unwrap(),
                            chunk_length,
                        }
                    }
                    VdafName::SumVec => {
                        let dp_strategy =
                            match (differential_privacy_strategy, differential_privacy_epsilon) {
                                (None, None) => dp_strategy::Prio3SumVec::NoDifferentialPrivacy,
                                (None, Some(_)) => {
                                    return Err(Error::Other(
                                        "missing differential-privacy-strategy".into(),
                                    ))
                                }
                                (Some(_), None) => {
                                    return Err(Error::Other(
                                        "missing differential-privacy-epsilon".into(),
                                    ))
                                }
                                (Some(DpStrategy::PureDpDiscreteLaplace), Some(epsilon)) => {
                                    dp_strategy::Prio3SumVec::PureDpDiscreteLaplace(
                                        PureDpDiscreteLaplace {
                                            budget: PureDpBudget {
                                                epsilon: float_to_biguint_ratio(epsilon)
                                                    .ok_or_else(|| {
                                                        Error::Other("invalid epsilon".into())
                                                    })?,
                                            },
                                        },
                                    )
                                }
                            };
                        Vdaf::SumVec(SumVec::new(
                            bits.unwrap(),
                            length.unwrap(),
                            chunk_length,
                            dp_strategy,
                        ))
                    }
                };

                let time_precision_seconds = time_precision.as_secs();
                let batch_time_window_size_seconds =
                    batch_time_window_size.map(|window| window.as_secs());

                let task = NewTask {
                    name,
                    leader_aggregator_id,
                    helper_aggregator_id,
                    vdaf,
                    min_batch_size,
                    max_batch_size,
                    batch_time_window_size_seconds,
                    time_precision_seconds,
                    collector_credential_id,
                };

                output.display(client.create_task(account_id, task).await?)
            }

            TaskAction::Rename { task_uuid, name } => {
                output.display(client.rename_task(&task_uuid, &name).await?)
            }

            TaskAction::CollectorAuthTokens { task_uuid } => {
                output.display(client.task_collector_auth_tokens(&task_uuid).await?)
            }
            TaskAction::Delete { task_uuid, force } => {
                if force {
                    client.force_delete_task(&task_uuid).await?
                } else {
                    client.delete_task(&task_uuid).await?
                }
            }
            TaskAction::SetExpiration {
                task_uuid,
                expiration,
                now,
            } => {
                let expiration = if now {
                    Some(OffsetDateTime::now_utc())
                } else {
                    expiration.map(|e| OffsetDateTime::from(Into::<SystemTime>::into(e)))
                };
                output.display(
                    client
                        .set_task_expiration(&task_uuid, expiration.as_ref())
                        .await?,
                )
            }
        }

        Ok(())
    }
}

fn float_to_biguint_ratio(value: f64) -> Option<Ratio<BigUint>> {
    let signed_ratio = Ratio::from_float(value)?;
    let unsigned_ratio = Ratio::new(
        signed_ratio.numer().clone().try_into().ok()?,
        signed_ratio.denom().clone().try_into().ok()?,
    );
    Some(unsigned_ratio)
}
