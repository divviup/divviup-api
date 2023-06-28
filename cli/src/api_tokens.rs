use crate::{output, CliResult};
use clap::Subcommand;
use divviup_client::{DivviupClient, Uuid};

#[derive(Subcommand, Debug)]
pub enum ApiTokenAction {
    List,
    Create,
    Delete { api_token_id: Uuid },
}

impl ApiTokenAction {
    pub async fn run(self, account_id: Uuid, client: DivviupClient) -> CliResult {
        match self {
            ApiTokenAction::List => {
                output(client.api_tokens(account_id).await?);
                Ok(())
            }

            ApiTokenAction::Create => {
                output(client.create_api_token(account_id).await?);
                Ok(())
            }

            ApiTokenAction::Delete { api_token_id } => {
                client.delete_api_token(api_token_id).await?;
                Ok(())
            }
        }
    }
}
