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
    pub skip_app_compilation: bool,
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

fn var<T: FromStr>(name: &'static str) -> Result<T, ApiConfigError> {
    let format = std::any::type_name::<T>();
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

fn var_optional<T: FromStr>(name: &'static str, default: T) -> Result<T, ApiConfigError> {
    match var(name) {
        Err(ApiConfigError::MissingEnvVar(_)) => Ok(default),
        other => other,
    }
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, ApiConfigError> {
        Ok(Self {
            database_url: var("DATABASE_URL")?,
            session_secret: var("SESSION_SECRET")?,
            api_url: var("API_URL")?,
            auth_client_id: var("AUTH_CLIENT_ID")?,
            auth_client_secret: var("AUTH_CLIENT_SECRET")?,
            auth_audience: var("AUTH_AUDIENCE")?,
            app_url: var("APP_URL")?,
            auth_url: var("AUTH_URL")?,
            aggregator_dap_url: var("AGGREGATOR_DAP_URL")?,
            aggregator_api_url: var("AGGREGATOR_API_URL")?,
            aggregator_secret: var("AGGREGATOR_SECRET")?,
            prometheus_host: var_optional("OTEL_EXPORTER_PROMETHEUS_HOST", "localhost".into())?,
            prometheus_port: var_optional("OTEL_EXPORTER_PROMETHEUS_PORT", 9464)?,
            postmark_token: var("POSTMARK_TOKEN")?,
            email_address: var("EMAIL_ADDRESS")?,
            postmark_url: Url::parse("https://api.postmarkapp.com").unwrap(),
            skip_app_compilation: var_optional("SKIP_APP_COMPILATION", false)?,
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
