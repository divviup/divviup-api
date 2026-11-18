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
use divviup_client::{CodecError, DivviupClient, Url, Uuid};
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
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};
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
    fn client(&self) -> CliResult<DivviupClient> {
        let http_client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(divviup_client::Error::from)?;
        let mut client = DivviupClient::new(self.token.clone(), http_client);
        client.set_url(self.url.clone());
        Ok(client)
    }

    pub async fn run(self) -> ExitCode {
        let client = match self.client() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        };
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

#[tokio::main]
pub async fn main() -> ExitCode {
    let tracing_filter = EnvFilter::from_default_env();
    let layer = tracing_subscriber::fmt::layer().pretty();
    let subscriber = Registry::default().with(layer.with_filter(tracing_filter));
    tracing::subscriber::set_global_default(subscriber)
        .expect("global default tracing subscriber was already set");

    tracing_log::LogTracer::init().expect("global logger was already set");

    ClientBin::parse().run().await
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
    use std::assert_matches;

    use clap::{CommandFactory, Parser};

    use crate::{
        tasks::{TaskAction, VdafName},
        ClientBin, Resource,
    };

    #[test]
    fn debug_assert() {
        ClientBin::command().debug_assert();
    }

    #[test]
    fn create_task_args() {
        // This test case is adapted from the Docker Compose demo script CI job.

        let args = [
            "divviup",
            "--token=DUATAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "task",
            "create",
            "--name=name",
            "--leader-aggregator-id=00000000-0000-0000-0000-000000000000",
            "--helper-aggregator-id=00000000-0000-0000-0000-000000000000",
            "--collector-credential-id=00000000-0000-0000-0000-000000000000",
            "--vdaf=histogram",
            "--categorical-buckets=0,1,2,3,4,5,6,7,8,9,10",
            "--min-batch-size=100",
            "--max-batch-size=200",
            "--time-precision=60sec",
        ];
        let parsed = match ClientBin::try_parse_from(args) {
            Ok(parsed) => parsed,
            Err(e) => panic!("{e}"),
        };

        let Resource::Task(TaskAction::Create {
            vdaf,
            min_batch_size,
            max_batch_size,
            time_precision,
            categorical_buckets,
            ..
        }) = parsed.command
        else {
            panic!("wrong command: {:?}", parsed.command);
        };

        assert_matches!(vdaf, VdafName::Histogram);
        assert_eq!(min_batch_size, 100);
        assert_eq!(max_batch_size, Some(200));
        assert_eq!(time_precision.as_secs(), 60);
        assert_matches!(categorical_buckets, Some(_));
    }

    #[test]
    fn set_expiration_args() {
        let args = [
            "divviup",
            "--token=DUATAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "task",
            "set-expiration",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "1970-01-01T00:00:00Z",
        ];
        let parsed = match ClientBin::try_parse_from(args) {
            Ok(parsed) => parsed,
            Err(e) => panic!("{e}"),
        };

        let Resource::Task(TaskAction::SetExpiration { expiration, .. }) = parsed.command else {
            panic!("wrong command: {:?}", parsed.command);
        };
        let expiration = expiration.unwrap();
        assert_eq!(expiration.to_string(), "1970-01-01T00:00:00Z");
    }
}
