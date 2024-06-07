#![forbid(unsafe_code)]
#![deny(
    clippy::dbg_macro,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

mod accounts;
mod aggregators;
mod api_tokens;
mod collector_credentials;
mod dap_client;
mod memberships;
mod tasks;

use accounts::AccountAction;
use aggregators::AggregatorAction;
use api_tokens::ApiTokenAction;
use clap::{Parser, Subcommand, ValueEnum};
use collector_credentials::CollectorCredentialAction;
use colored::Colorize;
use const_format::concatcp;
use dap_client::DapClientAction;
use divviup_client::{Client, CodecError, DivviupClient, HeaderValue, KnownHeaderName, Url, Uuid};
use memberships::MembershipAction;
use serde::Serialize;
use std::{
    fmt::{Debug, Display},
    future::Future,
    io::IsTerminal,
    pin::Pin,
    process::ExitCode,
};
use tasks::TaskAction;
use trillium_rustls::RustlsConfig;
use trillium_tokio::ClientConfig;

pub const USER_AGENT: &str = concatcp!(
    "divviup-cli/",
    env!("CARGO_PKG_VERSION"),
    " ",
    divviup_client::USER_AGENT
);

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
    #[arg(short, long, env = "DIVVIUP_TOKEN", hide_env_values = true)]
    token: String,

    #[arg(short, long, env = "DIVVIUP_API_URL", default_value = divviup_client::DEFAULT_URL)]
    url: Url,

    #[arg(short, long, env = "DIVVIUP_ACCOUNT_ID")]
    account_id: Option<Uuid>,

    #[arg(short, long, default_value_t)]
    output: Output,

    #[command(subcommand)]
    command: Resource,
}

#[derive(Subcommand, Debug)]
enum Resource {
    /// manage accounts
    #[command(subcommand)]
    Account(AccountAction),

    /// secret keys used to access the divviup api programmatically
    #[command(subcommand)]
    ApiToken(ApiTokenAction),

    /// privacy-preserving metrics in the divviup system
    #[command(subcommand)]
    Task(TaskAction),

    /// DAP client to upload metrics
    #[command(subcommand)]
    DapClient(DapClientAction),

    /// dap servers that have been paired to divviup for your account
    #[command(subcommand)]
    Aggregator(AggregatorAction),

    /// collaborate with others and manage access
    #[command(subcommand)]
    Membership(MembershipAction),

    /// manage asymmetrical encryption keys for collecting task aggregates later
    #[command(subcommand)]
    CollectorCredential(CollectorCredentialAction),
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

    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    CodecError(#[from] CodecError),

    #[error("{0:?}")]
    Anyhow(#[from] anyhow::Error),
}

pub type CliResult<T = ()> = Result<T, Error>;

impl ClientBin {
    fn client(&self) -> DivviupClient {
        let mut client = DivviupClient::new(
            self.token.clone(),
            Client::new(RustlsConfig::<ClientConfig>::default()).with_default_pool(),
        )
        .with_header(KnownHeaderName::UserAgent, HeaderValue::from(USER_AGENT));
        client.set_url(self.url.clone());
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
    // Choose aws-lc-rs as the default rustls crypto provider. This is what's currently enabled by
    // the default Cargo feature. Specifying a default provider here prevents runtime errors if
    // another dependency also enables the ring feature.
    let _ = trillium_rustls::rustls::crypto::aws_lc_rs::default_provider().install_default();

    env_logger::init();
    let args = ClientBin::parse();
    trillium_tokio::block_on(async move { args.run().await })
}

type DetermineAccountId = Pin<Box<dyn Future<Output = Result<Uuid, Error>> + Send + 'static>>;

impl Resource {
    pub async fn run(
        self,
        account_id: Option<Uuid>,
        client: DivviupClient,
        output: Output,
    ) -> Result<(), Error> {
        let account_id = {
            let client = client.clone();
            Box::pin(async move {
                if let Some(account_id) = account_id {
                    Ok(account_id)
                } else {
                    let accounts = client.accounts().await?;
                    if accounts.len() == 1 {
                        let id = accounts[0].id;
                        Ok(id)
                    } else {
                        Err(Error::CouldNotDetermineAccountId)
                    }
                }
            })
        };

        match self {
            Resource::Account(action) => action.run(account_id, client, output).await,
            Resource::ApiToken(action) => action.run(account_id, client, output).await,
            Resource::Task(action) => action.run(account_id, client, output).await,
            Resource::DapClient(action) => action.run(client).await,
            Resource::Aggregator(action) => action.run(account_id, client, output).await,
            Resource::Membership(action) => action.run(account_id, client, output).await,
            Resource::CollectorCredential(action) => action.run(account_id, client, output).await,
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
