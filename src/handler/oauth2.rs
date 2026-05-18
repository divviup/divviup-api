use crate::{clients::HttpClient, handler::Error, Config, User, USER_SESSION_KEY};
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    HttpClientError, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RequestTokenError, Scope,
    StandardErrorResponse, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use url::Url;

/// Type alias for an oauth2::Client once we've finished configuring it in `OauthClient::new`.
pub type ConfiguredOauthClient = BasicClient<
    EndpointSet,    // HasAuthURL
    EndpointNotSet, // HasDeviceAuthURL
    EndpointNotSet, // HasIntrospectionURL
    EndpointNotSet, // HasRevocationURL
    EndpointSet,    // HasTokenURL
>;

#[derive(Clone, Debug)]
pub struct Oauth2Config {
    pub authorize_url: Url,
    pub token_url: Url,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: Url,
    pub base_url: Url,
    pub audience: String,
    pub http_client: HttpClient,
}

const PKCE_SESSION_KEY: &str = "pkce_verifier";
const CSRF_SESSION_KEY: &str = "csrf_token";

/// `GET /login` — start the OAuth flow, or short-circuit to the app if the
/// user is already logged in.
pub async fn redirect(
    State(oauth_client): State<OauthClient>,
    State(config): State<Arc<Config>>,
    session: Session,
    user: Option<User>,
) -> Result<Response, Error> {
    if user.is_some() {
        return Ok(found_redirect(config.app_url.as_ref()));
    }

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = oauth_client
        .oauth2_client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(String::from("openid")))
        .add_scope(Scope::new(String::from("profile")))
        .add_scope(Scope::new(String::from("email")))
        .set_pkce_challenge(pkce_challenge)
        .url();

    session.insert(PKCE_SESSION_KEY, pkce_verifier).await?;
    session.insert(CSRF_SESSION_KEY, csrf_token).await?;

    Ok(found_redirect(auth_url.as_str()))
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

/// `GET /callback` — exchange the authorization code for tokens, then stash
/// the user in the session and redirect to the app.
pub async fn callback(
    State(oauth_client): State<OauthClient>,
    State(config): State<Arc<Config>>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Result<Response, Error> {
    let auth_code = params
        .code
        .filter(|s| !s.is_empty())
        .map(AuthorizationCode::new)
        .ok_or(Error::CallbackMissingCode)?;

    let pkce_verifier: PkceCodeVerifier = session
        .remove(PKCE_SESSION_KEY)
        .await?
        .ok_or(Error::CallbackMissingPkce)?;

    let session_csrf: Option<String> = session.remove(CSRF_SESSION_KEY).await?;
    match (session_csrf, &params.state) {
        (Some(a), Some(b)) if a == *b => {}
        _ => return Err(Error::CallbackCsrfMismatch),
    }

    let user = oauth_client
        .exchange_code_for_user(auth_code, pkce_verifier)
        .await?;

    session.insert(USER_SESSION_KEY, user).await?;

    Ok(found_redirect(config.app_url.as_ref()))
}

/// `GET /logout` — destroy the session and redirect to Auth0's logout URL so
/// the IdP session is also cleared.
pub async fn logout(
    State(config): State<Arc<Config>>,
    session: Session,
) -> Result<Response, Error> {
    session.flush().await?;

    let mut logout_url = config.auth_url.join("/v2/logout")?;
    logout_url.query_pairs_mut().extend_pairs([
        ("client_id", &*config.auth_client_id),
        ("returnTo", config.app_url.as_ref()),
    ]);

    Ok(found_redirect(logout_url.as_ref()))
}

fn found_redirect(location: &str) -> Response {
    (
        StatusCode::FOUND,
        [(header::LOCATION, location.to_string())],
    )
        .into_response()
}

#[derive(thiserror::Error, Debug)]
pub enum OauthError {
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error("error response: {0}")]
    RequestTokenError(StandardErrorResponse<BasicErrorResponseType>),
    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
    #[error("Other error: {0}")]
    Other(String),
    #[error("expected a successful status, but found {0}")]
    UnexpectedStatus(StatusCode),
    #[error(transparent)]
    HttpCrateError(#[from] oauth2::http::Error),
}

impl
    From<
        RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    > for OauthError
{
    fn from(
        value: RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ) -> Self {
        match value {
            RequestTokenError::ServerResponse(server_response) => {
                OauthError::RequestTokenError(server_response)
            }
            RequestTokenError::Request(HttpClientError::Reqwest(e)) => OauthError::HttpError(*e),
            RequestTokenError::Request(HttpClientError::Http(e)) => OauthError::HttpCrateError(e),
            RequestTokenError::Request(other) => OauthError::Other(other.to_string()),
            RequestTokenError::Parse(error, _path) => OauthError::Serde(error.into_inner()),
            RequestTokenError::Other(s) => OauthError::Other(s),
        }
    }
}

impl From<OauthError> for Error {
    fn from(value: OauthError) -> Self {
        Self::Other(Arc::new(value))
    }
}

#[derive(Clone, Debug)]
pub struct OauthClient(Arc<OauthClientInner>);

#[derive(Debug)]
struct OauthClientInner {
    oauth_config: Oauth2Config,
    oauth2_client: ConfiguredOauthClient,
    reqwest_client: reqwest::Client,
}

impl OauthClient {
    async fn exchange_code_for_user(
        &self,
        auth_code: AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<User, OauthError> {
        let exchange = self
            .oauth2_client()
            .exchange_code(auth_code)
            .set_pkce_verifier(pkce_verifier)
            .add_extra_param("audience", &self.0.oauth_config.audience)
            .request_async(&self.0.reqwest_client)
            .await?;

        let userinfo_url = self.0.oauth_config.base_url.join("/userinfo")?;
        let response = self
            .0
            .oauth_config
            .http_client
            .get_url(userinfo_url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", exchange.access_token().secret()),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OauthError::UnexpectedStatus(response.status()));
        }

        Ok(response.json().await?)
    }

    pub fn new(config: &Oauth2Config) -> Self {
        let oauth2_client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(AuthUrl::from_url(config.authorize_url.clone()))
            .set_token_uri(TokenUrl::from_url(config.token_url.clone()))
            .set_redirect_uri(RedirectUrl::from_url(config.redirect_url.clone()));

        let reqwest_client = reqwest::Client::new();

        Self(Arc::new(OauthClientInner {
            oauth_config: config.clone(),
            oauth2_client,
            reqwest_client,
        }))
    }

    pub fn oauth2_client(&self) -> &ConfiguredOauthClient {
        &self.0.oauth2_client
    }
}
