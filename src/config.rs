use crate::{
    handler::oauth2::Oauth2Config,
    trace::{TokioConsoleConfig, TraceConfig},
    Crypter,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use email_address::EmailAddress;
use std::{
    any::type_name,
    collections::VecDeque,
    env::{self, VarError},
    error::Error,
    net::SocketAddr,
    str::FromStr,
};
use thiserror::Error;
use trillium_client::Client;
use url::Url;

const POSTMARK_URL: &str = "https://api.postmarkapp.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: Url,
    pub app_url: Url,
    pub auth_audience: String,
    pub auth_client_id: String,
    pub auth_client_secret: String,
    pub auth_url: Url,
    pub client: Client,
    pub crypter: Crypter,
    pub database_url: Url,
    pub email_address: EmailAddress,
    pub postmark_token: String,
    pub postmark_url: Url,
    /// The address to listen on for prometheus metrics and tracing configuration.
    pub monitoring_listen_address: SocketAddr,
    pub session_secrets: SessionSecrets,
    /// See [`TraceConfig::use_test_writer`].
    pub trace_use_test_writer: bool,
    /// See [`TraceConfig::force_json_writer`].
    pub trace_force_json_writer: bool,
    /// See [`TraceConfig::stackdriver_json_output`].
    pub trace_stackdriver_json_output: bool,
    /// See [`TraceConfig::chrome`].
    pub trace_chrome: bool,
    /// See [`TokioConsoleConfig::enabled`].
    pub tokio_console_enabled: bool,
    /// See [`TokioConsoleConfig::listen_address`].
    pub tokio_console_listen_address: SocketAddr,
    /// Enables refreshing upload metrics from Janus. Enabled by default.
    pub metrics_refresh_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureFlags {
    /// Enables refreshing upload metrics from Janus. Enabled by default.
    pub metrics_refresh_enabled: bool,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("environment variable `{0}` was not found.")]
    MissingEnvVar(&'static str),

    #[error("environment variable `{0}` found but was not unicode")]
    NotUnicode(&'static str),

    #[error("environment variable `{0}` was found, but was not a valid {1}:\n\t{2}\n")]
    InvalidEnvVarFormat(String, &'static str, Box<dyn Error + 'static>),

    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
}

fn var<T>(name: &'static str) -> Result<T, ConfigError>
where
    T: FromStr,
    <T as FromStr>::Err: Error + 'static,
{
    let format = type_name::<T>();
    env::var(name)
        .map_err(|error| match error {
            VarError::NotPresent => ConfigError::MissingEnvVar(name),
            VarError::NotUnicode(_) => ConfigError::NotUnicode(name),
        })
        .and_then(|input| {
            input
                .parse()
                .map_err(|e| ConfigError::InvalidEnvVarFormat(name.into(), format, Box::new(e)))
        })
}

fn var_optional<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    <T as FromStr>::Err: Error + 'static,
{
    match var(name) {
        Err(ConfigError::MissingEnvVar(_)) => Ok(default),
        other => other,
    }
}

#[cfg(not(feature = "api-mocks"))]
fn build_client() -> trillium_client::Client {
    use trillium_rustls::RustlsConfig;
    use trillium_tokio::ClientConfig;
    Client::new(RustlsConfig::default().with_tcp_config(ClientConfig::default()))
        .with_default_pool()
}

#[cfg(feature = "api-mocks")]
fn build_client() -> trillium_client::Client {
    use crate::api_mocks::ApiMocks;

    Client::new(trillium_testing::connector(ApiMocks::new(
        POSTMARK_URL,
        var::<Url>("AUTH_URL").unwrap().as_ref(),
    )))
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            api_url: var("API_URL")?,
            app_url: var("APP_URL")?,
            auth_audience: var("AUTH_AUDIENCE")?,
            auth_client_id: var("AUTH_CLIENT_ID")?,
            auth_client_secret: var("AUTH_CLIENT_SECRET")?,
            auth_url: var("AUTH_URL")?,
            client: build_client(),
            crypter: var("DATABASE_ENCRYPTION_KEYS")?,
            database_url: var("DATABASE_URL")?,
            email_address: var("EMAIL_ADDRESS")?,
            postmark_token: var("POSTMARK_TOKEN")?,
            postmark_url: Url::parse(POSTMARK_URL).unwrap(),
            monitoring_listen_address: var_optional(
                "MONITORING_LISTEN_ADDRESS",
                "127.0.0.1:9464".parse().unwrap(),
            )?,
            session_secrets: var("SESSION_SECRETS")?,
            trace_use_test_writer: false,
            trace_force_json_writer: var_optional("TRACE_FORCE_JSON_WRITER", false)?,
            trace_stackdriver_json_output: var_optional("TRACE_STACKDRIVER_JSON_OUTPUT", false)?,
            trace_chrome: var_optional("TRACE_CHROME", false)?,
            tokio_console_enabled: var_optional("TOKIO_CONSOLE_ENABLED", false)?,
            tokio_console_listen_address: var_optional(
                "TOKIO_CONSOLE_LISTEN_ADDRESS",
                "127.0.0.1:6669".parse().unwrap(),
            )?,
            metrics_refresh_enabled: var_optional("METRICS_REFRESH_ENABLED", true)?,
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

    pub fn trace_config(&self) -> TraceConfig {
        TraceConfig {
            use_test_writer: self.trace_use_test_writer,
            force_json_output: self.trace_force_json_writer,
            stackdriver_json_output: self.trace_stackdriver_json_output,
            tokio_console_config: TokioConsoleConfig {
                enabled: self.tokio_console_enabled,
                listen_address: Some(self.tokio_console_listen_address),
            },
            chrome: self.trace_chrome,
        }
    }

    pub fn feature_flags(&self) -> FeatureFlags {
        FeatureFlags {
            metrics_refresh_enabled: self.metrics_refresh_enabled,
        }
    }
}

impl AsRef<Client> for Config {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum SessionSecretsDecodeError {
    #[error("session secret must be at least 32 bytes after base64 decode")]
    TooShort,
    #[error("session secret must be base64url without padding")]
    Base64,
    #[error("at least one session secret must be provided")]
    Missing,
}

#[derive(Clone, Debug)]
pub struct SessionSecrets {
    pub current: Vec<u8>,
    pub older: Vec<Vec<u8>>,
}
impl From<Vec<u8>> for SessionSecrets {
    fn from(current: Vec<u8>) -> Self {
        Self {
            current,
            older: vec![],
        }
    }
}

impl FromStr for SessionSecrets {
    type Err = SessionSecretsDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut secret = s
            .split(',')
            .map(|s| URL_SAFE_NO_PAD.decode(s))
            .collect::<Result<VecDeque<Vec<u8>>, _>>()
            .map_err(|_| SessionSecretsDecodeError::Base64)?;
        if secret.iter().any(|x| x.len() < 32) {
            return Err(SessionSecretsDecodeError::TooShort);
        }
        let current = secret
            .pop_front()
            .ok_or(SessionSecretsDecodeError::Missing)?;
        let older = secret.into();
        Ok(Self { current, older })
    }
}
