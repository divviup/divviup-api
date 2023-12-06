pub mod aggregator_client;
pub mod auth0_client;
pub mod postmark_client;

pub use aggregator_client::AggregatorClient;
pub use auth0_client::Auth0Client;
use backoff::ExponentialBackoff;
pub use postmark_client::PostmarkClient;
use std::time::Duration;
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

impl ClientError {
    pub(crate) fn into_backoff(self) -> backoff::Error<Self> {
        match self {
            Self::HttpStatusNotSuccess {
                status: Some(status),
                ..
            } if (status.is_server_error() && status != Status::NotImplemented)
                || status == Status::TooManyRequests =>
            {
                backoff::Error::transient(self)
            }
            Self::Http(trillium_http::Error::Io(_) | trillium_http::Error::Closed) => {
                backoff::Error::transient(self)
            }
            _ => backoff::Error::permanent(self),
        }
    }
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

/// An [`ExponentialBackoff`] with parameters suitable for most HTTP requests. The parameters are
/// copied from the parameters used in the GCP Go SDK[1].
///
/// [1]: https://github.com/googleapis/gax-go/blob/fbaf9882acf3297573f3a7cb832e54c7d8f40635/v2/call_option.go#L120
pub(crate) fn http_request_exponential_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: Duration::from_secs(1),
        max_interval: Duration::from_secs(30),
        multiplier: 2.0,
        max_elapsed_time: Some(Duration::from_secs(600)),
        ..Default::default()
    }
}
