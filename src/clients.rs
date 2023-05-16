pub mod aggregator_client;
pub mod auth0_client;
pub mod postmark_client;

pub use aggregator_client::AggregatorClient;
pub use auth0_client::Auth0Client;
pub use postmark_client::PostmarkClient;
use trillium::{async_trait, Status};
use trillium_client::{ClientSerdeError, Conn};
use trillium_http::Method;
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("unexpected http status {method} {url} {status:?}: {body}")]
    HttpStatusNotSuccess {
        method: Method,
        url: Url,
        status: Option<Status>,
        body: String,
    },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] trillium_http::Error),

    #[error("{0}")]
    Other(String),
}

#[async_trait]
pub trait ClientConnExt: Sized {
    async fn success_or_client_error(self) -> Result<Self, ClientError>;
}
#[async_trait]
impl ClientConnExt for Conn {
    async fn success_or_client_error(self) -> Result<Self, ClientError> {
        match self.await?.success() {
            Ok(conn) => Ok(conn),
            Err(mut error) => {
                let status = error.status();
                let url = error.url().clone();
                let method = error.method();
                let body = error.response_body().await?;
                Err(ClientError::HttpStatusNotSuccess {
                    method,
                    url,
                    status,
                    body,
                })
            }
        }
    }
}

impl From<ClientSerdeError> for ClientError {
    fn from(value: ClientSerdeError) -> Self {
        match value {
            ClientSerdeError::HttpError(h) => h.into(),
            ClientSerdeError::JsonError(j) => j.into(),
        }
    }
}
