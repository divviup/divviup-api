pub mod aggregator_client;
pub mod auth0_client;
pub mod postmark_client;

pub use aggregator_client::AggregatorClient;
pub use auth0_client::Auth0Client;
pub use postmark_client::PostmarkClient;
use trillium::Status;
use trillium_client::ClientSerdeError;
use trillium_rustls::RustlsConnector;
use trillium_tokio::TcpConnector;

pub type ClientConnector = RustlsConnector<TcpConnector>;
pub type Conn<'a> = trillium_client::Conn<'a, ClientConnector>;
pub type Client = trillium_client::Client<ClientConnector>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("unexpected api client http status {status:?}: {body}")]
    HttpStatusNotSuccess {
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

impl From<ClientSerdeError> for ClientError {
    fn from(value: ClientSerdeError) -> Self {
        match value {
            ClientSerdeError::HttpError(h) => h.into(),
            ClientSerdeError::JsonError(j) => j.into(),
        }
    }
}
pub async fn expect_ok(conn: &mut Conn<'_>) -> Result<(), ClientError> {
    if conn.status().map_or(false, |s| s.is_success()) {
        Ok(())
    } else {
        let body = conn.response_body().read_string().await?;
        log::error!("{:?}: {body}", conn.status());
        Err(ClientError::HttpStatusNotSuccess {
            status: conn.status(),
            body,
        })
    }
}
