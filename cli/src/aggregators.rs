use crate::{CliResult, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, NewAggregator, Url, Uuid};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Role {
    Either,
    Leader,
    Helper,
}

impl From<Role> for divviup_client::Role {
    fn from(value: Role) -> Self {
        match value {
            Role::Either => Self::Either,
            Role::Leader => Self::Leader,
            Role::Helper => Self::Helper,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum AggregatorAction {
    List,
    Create {
        /// Possible roles for this aggregator
        ///
        /// Acceptable values: "Either", "Leader", or "Helper"
        #[arg(short, long)]
        role: Role,

        /// Human-readable identifier for this aggregator
        ///
        /// This can be changed later
        #[arg(short, long)]
        name: String,

        /// DAP URL for this aggregator
        ///
        /// must be https
        #[arg(short, long)]
        dap_url: Url,

        /// API URL for this aggregator
        ///
        /// must be https
        #[arg(short, long)]
        api_url: Url,

        /// bearer token for this aggregator
        #[arg(short, long)]
        bearer_token: String,
    },

    Rename {
        /// uuid for this aggregator
        aggregator_id: Uuid,
        /// new name
        name: String,
    },
}

impl AggregatorAction {
    pub(crate) async fn run(
        self,
        account_id: Uuid,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            AggregatorAction::List => output.display(client.aggregators(account_id).await?),

            AggregatorAction::Create {
                role,
                name,
                dap_url,
                api_url,
                bearer_token,
            } => output.display(
                client
                    .create_aggregator(
                        account_id,
                        NewAggregator {
                            role: role.into(),
                            name,
                            dap_url,
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
        }
        Ok(())
    }
}
