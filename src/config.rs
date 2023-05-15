use crate::handler::oauth2::Oauth2Config;
use email_address::EmailAddress;
use std::{env::VarError, str::FromStr};
use thiserror::Error;
use trillium_client::Client;
use trillium_rustls::RustlsConfig;
use trillium_tokio::ClientConfig;
use url::Url;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub session_secret: String,
    pub api_url: Url,
    pub app_url: Url,
    pub database_url: Url,
    pub auth_url: Url,
    pub auth_client_id: String,
    pub auth_client_secret: String,
    pub auth_audience: String,
    pub aggregator_dap_url: Url,
    pub aggregator_api_url: Url,
    pub aggregator_secret: String,
    pub prometheus_host: String,
    pub prometheus_port: u16,
    pub postmark_token: String,
    pub email_address: EmailAddress,
    pub postmark_url: Url,
    pub client: Client,
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ApiConfigError {
    #[error("environment variable `{0}` was not found.")]
    MissingEnvVar(&'static str),

    #[error("The environment variable `{0}` was found, but was not a valid `{1}`")]
    InvalidEnvVarFormat(&'static str, &'static str),

    #[error("could not join url to `{0:?}`")]
    InvalidUrl(#[from] url::ParseError),
}

fn var<T: FromStr>(name: &'static str, format: &'static str) -> Result<T, ApiConfigError> {
    std::env::var(name)
        .map_err(|error| match error {
            VarError::NotPresent => ApiConfigError::MissingEnvVar(name),
            VarError::NotUnicode(_) => ApiConfigError::InvalidEnvVarFormat(name, format),
        })
        .and_then(|input| {
            input
                .parse()
                .map_err(|_| ApiConfigError::InvalidEnvVarFormat(name, format))
        })
}

fn var_optional<T: FromStr + 'static>(
    name: &'static str,
    default: &'static str,
    format: &'static str,
) -> Result<T, ApiConfigError> {
    let input_res = std::env::var(name);
    let input = match &input_res {
        Ok(value) => value,
        Err(VarError::NotPresent) => default,
        Err(VarError::NotUnicode(_)) => {
            return Err(ApiConfigError::InvalidEnvVarFormat(name, format))
        }
    };
    input
        .parse()
        .map_err(|_| ApiConfigError::InvalidEnvVarFormat(name, format))
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiConfigError> {
        Ok(Self {
            database_url: var("DATABASE_URL", "url")?,
            session_secret: var("SESSION_SECRET", "string")?,
            api_url: var("API_URL", "url")?,
            auth_client_id: var("AUTH_CLIENT_ID", "string")?,
            auth_client_secret: var("AUTH_CLIENT_SECRET", "string")?,
            auth_audience: var("AUTH_AUDIENCE", "string")?,
            app_url: var("APP_URL", "url")?,
            auth_url: var("AUTH_URL", "url")?,
            aggregator_dap_url: var("AGGREGATOR_DAP_URL", "url")?,
            aggregator_api_url: var("AGGREGATOR_API_URL", "url")?,
            aggregator_secret: var("AGGREGATOR_SECRET", "string")?,
            prometheus_host: var_optional("OTEL_EXPORTER_PROMETHEUS_HOST", "localhost", "string")?,
            prometheus_port: var_optional(
                "OTEL_EXPORTER_PROMETHEUS_PORT",
                "9464",
                "16-bit number",
            )?,
            postmark_token: var("POSTMARK_TOKEN", "string")?,
            email_address: var("EMAIL_ADDRESS", "email")?,
            postmark_url: Url::parse("https://api.postmarkapp.com").unwrap(),
            client: Client::new(RustlsConfig::default().with_tcp_config(ClientConfig::default()))
                .with_default_pool(),
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
