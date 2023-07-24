mod accounts;
mod aggregators;
mod api_tokens;
mod memberships;
mod tasks;

use std::{
    fmt::{Debug, Display},
    io::IsTerminal,
    process::ExitCode,
};

use accounts::AccountAction;
use aggregators::AggregatorAction;
use api_tokens::ApiTokenAction;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use divviup_client::{Client, DivviupClient, Url, Uuid};
use memberships::MembershipAction;
use serde::Serialize;
use tasks::TaskAction;
use trillium_rustls::RustlsConfig;
use trillium_tokio::ClientConfig;

#[derive(ValueEnum, Debug, Default, Clone, Copy)]
enum Output {
    #[default]
    Json,
    Yaml,
    Text,
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Output::Json => "json",
            Output::Yaml => "yaml",
            Output::Text => "text",
        })
    }
}

impl Output {
    pub fn display<T>(self, t: T)
    where
        T: Debug + Serialize,
    {
        match self {
            Output::Text => {
                println!("{t:#?}");
            }

            Output::Json => {
                println!("{}", serde_json::to_string_pretty(&t).unwrap());
            }

            Output::Yaml => {
                println!("{}", serde_yaml::to_string(&t).unwrap());
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct ClientBin {
    #[arg(short, long, env)]
    token: String,

    #[arg(short, long, env = "API_URL")]
    url: Option<Url>,

    #[arg(short, long, env = "ACCOUNT_ID")]
    account_id: Option<Uuid>,

    #[arg(short, long, default_value_t)]
    output: Output,

    #[command(subcommand)]
    command: Resource,
}

#[derive(Subcommand, Debug)]
enum Resource {
    #[command(subcommand)]
    Account(AccountAction),

    #[command(subcommand)]
    ApiToken(ApiTokenAction),

    #[command(subcommand)]
    Task(TaskAction),

    #[command(subcommand)]
    Aggregator(AggregatorAction),

    #[command(subcommand)]
    Membership(MembershipAction),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] divviup_client::Error),

    #[error("account id could not be determined")]
    CouldNotDetermineAccountId,

    #[error(transparent)]
    Base64(#[from] base64::DecodeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type CliResult<T = ()> = Result<T, Error>;

impl ClientBin {
    fn client(&self) -> DivviupClient {
        let mut client = DivviupClient::new(
            self.token.clone(),
            Client::new(RustlsConfig::<ClientConfig>::default()).with_default_pool(),
        );
        if let Some(url) = self.url.clone() {
            client.set_url(url);
        }
        client
    }

    pub async fn run(self) -> ExitCode {
        let client = self.client();
        match self.command.run(self.account_id, client, self.output).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                if std::io::stdout().is_terminal() {
                    eprintln!("{}", e.to_string().red());
                } else {
                    eprintln!("{e}");
                }
                ExitCode::FAILURE
            }
        }
    }
}

pub fn main() -> ExitCode {
    env_logger::init();
    let args = ClientBin::parse();
    trillium_tokio::block_on(async move { args.run().await })
}

impl Resource {
    pub async fn run(
        self,
        account_id: Option<Uuid>,
        client: DivviupClient,
        output: Output,
    ) -> Result<(), Error> {
        let (accounts, account_id) = match account_id {
            None => {
                let accounts = client.accounts().await?;
                if accounts.len() == 1 {
                    let id = accounts[0].id;
                    (Some(accounts), Some(id))
                } else {
                    (Some(accounts), None)
                }
            }
            Some(account_id) => (None, Some(account_id)),
        };

        let Some(account_id) = account_id else {
            return Err(Error::CouldNotDetermineAccountId);
        };

        match self {
            Resource::Account(action) => action.run(account_id, client, accounts, output).await,
            Resource::ApiToken(action) => action.run(account_id, client, output).await,
            Resource::Task(action) => action.run(account_id, client, output).await,
            Resource::Aggregator(action) => action.run(account_id, client, output).await,
            Resource::Membership(action) => action.run(account_id, client, output).await,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debug_assert() {
        use clap::CommandFactory;
        super::ClientBin::command().debug_assert();
    }
}
