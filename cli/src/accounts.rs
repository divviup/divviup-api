use crate::{CliResult, DetermineAccountId, Output};
use clap::Subcommand;
use divviup_client::DivviupClient;

#[derive(Subcommand, Debug)]
pub enum AccountAction {
    /// List all accounts
    List,

    #[cfg(feature = "admin")]
    /// Create a new account (ADMIN)
    Create { name: String },

    /// Rename the target account
    Rename { name: String },
}

impl AccountAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match &self {
            AccountAction::List => output.display(client.accounts().await?),

            #[cfg(feature = "admin")]
            AccountAction::Create { name } => output.display(client.create_account(name).await?),

            AccountAction::Rename { name } => {
                output.display(client.rename_account(account_id.await?, name).await?)
            }
        }

        Ok(())
    }
}
