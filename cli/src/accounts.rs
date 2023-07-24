use crate::CliResult;
use clap::Subcommand;
use divviup_client::{Account, DivviupClient, Uuid};

use super::Output;

#[derive(Subcommand, Debug)]
pub enum AccountAction {
    List,
    Create { name: String },
    Rename { name: String },
}

impl AccountAction {
    pub(crate) async fn run(
        self,
        account_id: Uuid,
        client: DivviupClient,
        accounts: Option<Vec<Account>>,
        output: Output,
    ) -> CliResult {
        match &self {
            AccountAction::List => output.display(match accounts {
                Some(accounts) => accounts,
                None => client.accounts().await?,
            }),

            AccountAction::Create { name } => output.display(client.create_account(name).await?),

            AccountAction::Rename { name } => {
                output.display(client.rename_account(account_id, name).await?)
            }
        }

        Ok(())
    }
}
