use crate::{CliResult, DetermineAccountId, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};

#[derive(Subcommand, Debug)]
pub enum ApiTokenAction {
    /// list all api tokens for to the target account
    List,

    /// create a new api token attached to the target account
    Create,

    /// deletes an api token by id
    Delete { api_token_id: Uuid },
}

impl ApiTokenAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            ApiTokenAction::List => {
                output.display(client.api_tokens(account_id.await?).await?);
            }

            ApiTokenAction::Create => {
                output.display(client.create_api_token(account_id.await?).await?);
            }

            ApiTokenAction::Delete { api_token_id } => {
                client.delete_api_token(api_token_id).await?;
            }
        }
        Ok(())
    }
}
