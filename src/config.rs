use crate::handler::oauth2::Oauth2Config;
use email_address::EmailAddress;
use std::{env::VarError, str::FromStr};
use thiserror::Error;
use trillium_client::Client;
use trillium_rustls::RustlsConfig;
use trillium_tokio::ClientConfig;
use url::Url;

const POSTMARK_URL: &str = "https://api.postmarkapp.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub session_secret: String,
    pub api_url: Url,
    pub app_url: Url,
    pub database_url: Url,
    pub auth_url: Url,
    pub auth_client_id: String,
    pub auth_client_secret: String,
    pub auth_audience: String,
    pub prometheus_host: String,
    pub prometheus_port: u16,
    pub postmark_token: String,
    pub email_address: EmailAddress,
    pub postmark_url: Url,
    pub client: Client,
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ConfigError {
    #[error("environment variable `{0}` was not found.")]
    MissingEnvVar(&'static str),

    #[error("The environment variable `{0}` was found, but was not a valid `{1}`")]
    InvalidEnvVarFormat(&'static str, &'static str),

    #[error("could not join url to `{0:?}`")]
    InvalidUrl(#[from] url::ParseError),
}

fn var<T: FromStr>(name: &'static str) -> Result<T, ConfigError> {
    let format = std::any::type_name::<T>();
    std::env::var(name)
        .map_err(|error| match error {
            VarError::NotPresent => ConfigError::MissingEnvVar(name),
            VarError::NotUnicode(_) => ConfigError::InvalidEnvVarFormat(name, format),
        })
        .and_then(|input| {
            input
                .parse()
                .map_err(|_| ConfigError::InvalidEnvVarFormat(name, format))
        })
}

fn var_optional<T: FromStr>(name: &'static str, default: T) -> Result<T, ConfigError> {
    match var(name) {
        Err(ConfigError::MissingEnvVar(_)) => Ok(default),
        other => other,
    }
}

#[cfg(not(feature = "api-mocks"))]
fn build_client() -> trillium_client::Client {
    Client::new(RustlsConfig::default().with_tcp_config(ClientConfig::default()))
        .with_default_pool()
}

#[cfg(feature = "api-mocks")]
fn build_client() -> trillium_client::Client {
    Client::new(trillium_testing::connector(
        crate::api_mocks::ApiMocks::new(POSTMARK_URL, var::<Url>("AUTH_URL").unwrap().as_ref()),
    ))
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: var("DATABASE_URL")?,
            session_secret: var("SESSION_SECRET")?,
            api_url: var("API_URL")?,
            auth_client_id: var("AUTH_CLIENT_ID")?,
            auth_client_secret: var("AUTH_CLIENT_SECRET")?,
            auth_audience: var("AUTH_AUDIENCE")?,
            app_url: var("APP_URL")?,
            auth_url: var("AUTH_URL")?,
            prometheus_host: var_optional("OTEL_EXPORTER_PROMETHEUS_HOST", "localhost".into())?,
            prometheus_port: var_optional("OTEL_EXPORTER_PROMETHEUS_PORT", 9464)?,
            postmark_token: var("POSTMARK_TOKEN")?,
            email_address: var("EMAIL_ADDRESS")?,
            postmark_url: Url::parse(POSTMARK_URL).unwrap(),
            client: build_client(),
        })
    }

    pub fn oauth_config(&self) -> Oauth2Config {
        Oauth2Config {
            redirect_url: self.api_url.join("/callback").unwrap(),
            authorize_url: self.auth_url.join("/authorize").unwrap(),
            token_url: self.auth_url.join("/oauth/token").unwrap(),
            client_id: self.auth_client_id.clone(),
            client_secret: self.auth_client_secret.clone(),
            base_url: self.auth_url.clone(),
            audience: self.auth_audience.clone(),
            http_client: self.client.clone(),
        }
    }
}

impl AsRef<Client> for Config {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}
