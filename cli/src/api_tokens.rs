use crate::{CliResult, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};

#[derive(Subcommand, Debug)]
pub enum ApiTokenAction {
    List,
    Create,
    Delete { api_token_id: Uuid },
}

impl ApiTokenAction {
    pub(crate) async fn run(
        self,
        account_id: Uuid,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        match self {
            ApiTokenAction::List => {
                output.display(client.api_tokens(account_id).await?);
            }

            ApiTokenAction::Create => {
                output.display(client.create_api_token(account_id).await?);
            }

            ApiTokenAction::Delete { api_token_id } => {
                client.delete_api_token(api_token_id).await?;
            }
        }
        Ok(())
    }
}
