use crate::{CliResult, DetermineAccountId, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, NewAggregator, Url, Uuid};

#[derive(Subcommand, Debug)]
pub enum AggregatorAction {
    /// List all aggregators for the target account
    List {
        /// list only shared aggregators
        #[arg(short, long)]
        shared: bool,
    },

    /// Create a new aggregator
    Create {
        /// Human-readable identifier for this aggregator
        ///
        /// This can be changed later
        #[arg(short, long)]
        name: String,

        /// API URL for this aggregator
        ///
        /// must be https
        #[arg(short, long)]
        api_url: Url,

        /// bearer token for this aggregator
        #[arg(short, long)]
        bearer_token: String,

        #[arg(short, long)]
        #[cfg(feature = "admin")]
        /// create an aggregator that is usable by all accounts (ADMIN)
        shared: bool,

        #[arg(short, long, requires = "shared")]
        #[cfg(feature = "admin")]
        /// create an aggregator that is considered first party (ADMIN)
        first_party: bool,
    },

    /// Change the display name of an aggregator
    Rename {
        /// uuid for this aggregator
        aggregator_id: Uuid,

        /// new name
        name: String,
    },

    /// Rotate the bearer token for an aggregator
    RotateBearerToken {
        /// uuid for this aggregator
        aggregator_id: Uuid,

        /// new bearer token for this aggregator
        bearer_token: String,
    },
}

impl AggregatorAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            AggregatorAction::List { shared: true } => {
                output.display(client.shared_aggregators().await?)
            }

            AggregatorAction::List { shared: false } => {
                let account_id = account_id.await?;
                output.display(client.aggregators(account_id).await?)
            }

            #[cfg(feature = "admin")]
            AggregatorAction::Create {
                name,
                api_url,
                bearer_token,
                first_party,
                shared: true,
            } => output.display(
                client
                    .create_shared_aggregator(divviup_client::NewSharedAggregator {
                        name,
                        api_url,
                        bearer_token,
                        is_first_party: first_party,
                    })
                    .await?,
            ),

            AggregatorAction::Create {
                name,
                api_url,
                bearer_token,
                #[cfg(feature = "admin")]
                    shared: false,
                ..
            } => output.display(
                client
                    .create_aggregator(
                        account_id.await?,
                        NewAggregator {
                            name,
                            api_url,
                            bearer_token,
                        },
                    )
                    .await?,
            ),

            AggregatorAction::Rename {
                aggregator_id,
                name,
            } => output.display(client.rename_aggregator(aggregator_id, &name).await?),

            AggregatorAction::RotateBearerToken {
                aggregator_id,
                bearer_token,
            } => output.display(
                client
                    .rotate_aggregator_bearer_token(aggregator_id, &bearer_token)
                    .await?,
            ),
        }
        Ok(())
    }
}
