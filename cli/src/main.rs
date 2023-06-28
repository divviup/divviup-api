mod accounts;
mod aggregators;
mod api_tokens;
mod memberships;
mod tasks;

use accounts::AccountAction;
use aggregators::AggregatorAction;
use api_tokens::ApiTokenAction;
use clap::{Parser, Subcommand};
use divviup_client::{Client, DivviupClient, Url, Uuid};
use memberships::MembershipAction;
use tasks::TaskAction;
use trillium_rustls::RustlsConfig;
use trillium_tokio::ClientConfig;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct ClientBin {
    #[arg(short, long, env)]
    token: String,

    #[arg(short, long, env = "API_URL")]
    url: Option<Url>,

    #[arg(short, long, env = "ACCOUNT_ID")]
    account_id: Option<Uuid>,

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

    pub async fn run(self) -> Result<(), Error> {
        let client = self.client();
        self.command.run(self.account_id, client).await
    }
}

pub fn main() -> Result<(), Error> {
    let args = ClientBin::parse();
    trillium_tokio::block_on(async move { args.run().await })
}

impl Resource {
    pub async fn run(self, account_id: Option<Uuid>, client: DivviupClient) -> Result<(), Error> {
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
            Resource::Account(action) => action.run(account_id, client, accounts).await,
            Resource::ApiToken(action) => action.run(account_id, client).await,
            Resource::Task(action) => action.run(account_id, client).await,
            Resource::Aggregator(action) => action.run(account_id, client).await,
            Resource::Membership(action) => action.run(account_id, client).await,
        }
    }
}

pub fn output<T>(t: T)
where
    T: std::fmt::Debug + serde::Serialize,
{
    println!("{:#?}", t);
}
