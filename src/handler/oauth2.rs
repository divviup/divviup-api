use crate::{User, USER_SESSION_KEY};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType},
    AsyncHttpClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, HttpRequest, HttpResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    RequestTokenError, Scope, StandardErrorResponse, TokenResponse, TokenUrl,
};
use querystrong::QueryStrong;
use std::{future::Future, pin::Pin, sync::Arc};
use trillium::{conn_try, conn_unwrap, Conn, KnownHeaderName::Authorization, Status};
use trillium_client::{Client, ClientSerdeError};
use trillium_http::Headers;
use trillium_redirect::RedirectConnExt;
use trillium_sessions::SessionConnExt;
use url::Url;

/// Type alias for an oauth2::Client once we've finished configuring it in `OauthClient::new`.
/// Crate oauth's guide to upgrading to 0.5 recommends defining this kind of alias:
/// https://github.com/ramosbugs/oauth2-rs/blob/main/UPGRADE.md#add-typestate-generic-types-to-client
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
    pub http_client: Client,
}

pub async fn redirect(conn: Conn) -> Conn {
    let client: &OauthClient = conn.state().unwrap();

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .oauth2_client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(String::from("openid")))
        .add_scope(Scope::new(String::from("profile")))
        .add_scope(Scope::new(String::from("email")))
        .set_pkce_challenge(pkce_challenge)
        .url();

    conn.redirect(auth_url.to_string())
        .with_session("pkce_verifier", pkce_verifier)
        .with_session("csrf_token", csrf_token)
}

pub async fn callback(conn: Conn) -> Conn {
    let qs = QueryStrong::parse(conn.querystring()).unwrap_or_default();

    let Some(auth_code) = qs
        .get_str("code")
        .map(|c| AuthorizationCode::new(String::from(c)))
    else {
        return conn
            .with_body("expected code query param")
            .with_status(Status::Forbidden)
            .halt();
    };

    let Some(pkce_verifier) = conn.session().get("pkce_verifier") else {
        return conn
            .with_body("expected pkce verifier in session")
            .with_status(Status::Forbidden)
            .halt();
    };

    let session_csrf: Option<String> = conn.session().get("csrf_token");
    let qs_csrf = qs.get_str("state");

    if session_csrf.is_none() || qs_csrf != session_csrf.as_deref() {
        return conn
            .with_body("csrf mismatch or missing")
            .with_status(Status::Forbidden)
            .halt();
    }

    let client = conn_unwrap!(conn.state::<OauthClient>(), conn);
    let user = conn_try!(
        client
            .exchange_code_for_user(auth_code, pkce_verifier)
            .await,
        conn
    );

    conn.with_session(USER_SESSION_KEY, user)
}

#[derive(thiserror::Error, Debug)]
enum OauthError {
    #[error(transparent)]
    HttpError(#[from] trillium_client::Error),
    #[error(transparent)]
    InvalidStatusCode(#[from] oauth2::http::status::InvalidStatusCode),
    #[error(transparent)]
    HeaderConversionError(#[from] trillium_http::http_compat1::HeaderConversionError),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
    #[error("error response: {0}")]
    RequestTokenError(StandardErrorResponse<BasicErrorResponseType>),
    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
    #[error("Other error: {0}")]
    Other(String),
    #[error("expected a successful status, but found {0:?}")]
    UnexpectedStatus(Option<Status>),
    #[error(transparent)]
    HttpCrateError(#[from] oauth2::http::Error),
}

impl From<RequestTokenError<OauthError, StandardErrorResponse<BasicErrorResponseType>>>
    for OauthError
{
    fn from(
        value: RequestTokenError<OauthError, StandardErrorResponse<BasicErrorResponseType>>,
    ) -> Self {
        match value {
            RequestTokenError::ServerResponse(server_response) => {
                OauthError::RequestTokenError(server_response)
            }
            RequestTokenError::Request(e) => e,
            RequestTokenError::Parse(error, _path) => OauthError::Serde(error.into_inner()),
            RequestTokenError::Other(s) => OauthError::Other(s),
        }
    }
}

impl From<ClientSerdeError> for OauthError {
    fn from(value: ClientSerdeError) -> Self {
        match value {
            ClientSerdeError::HttpError(e) => OauthError::HttpError(e),
            ClientSerdeError::JsonError(e) => OauthError::Serde(e),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OauthClient(Arc<OauthClientInner>);

#[derive(Debug)]
struct OauthClientInner {
    oauth_config: Oauth2Config,
    oauth2_client: ConfiguredOauthClient,
}

impl OauthClient {
    async fn exchange_code_for_user(
        &self,
        auth_code: AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<User, OauthError> {
        let http_client = self.http_client().clone();
        let exchange = self
            .oauth2_client()
            .exchange_code(auth_code)
            .set_pkce_verifier(pkce_verifier)
            .add_extra_param("audience", &self.0.oauth_config.audience)
            .request_async(&ClientWrapper(http_client))
            .await?;

        let mut client_conn = self
            .http_client()
            .get(self.0.oauth_config.base_url.join("/userinfo")?)
            .with_request_header(
                Authorization,
                format!("Bearer {}", exchange.access_token().secret()),
            )
            .await?;
        if !client_conn
            .status()
            .as_ref()
            .map(Status::is_success)
            .unwrap_or_default()
        {
            return Err(OauthError::UnexpectedStatus(client_conn.status()));
        }

        Ok(client_conn.response_json().await?)
    }

    pub fn new(config: &Oauth2Config) -> Self {
        let oauth2_client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(AuthUrl::from_url(config.authorize_url.clone()))
            .set_token_uri(TokenUrl::from_url(config.token_url.clone()))
            .set_redirect_uri(RedirectUrl::from_url(config.redirect_url.clone()));

        Self(Arc::new(OauthClientInner {
            oauth_config: config.clone(),
            oauth2_client,
        }))
    }

    pub fn oauth2_client(&self) -> &ConfiguredOauthClient {
        &self.0.oauth2_client
    }

    pub fn http_client(&self) -> &Client {
        &self.0.oauth_config.http_client
    }
}

// Wraps a [`trillium_client::Client`] so we can implement [`oauth2::AsyncHttpClient`] on it, as
// otherwise the orphan rule would forbid this.
struct ClientWrapper(Client);

// Inspired by the impls `oauth2` provides for `reqwest::Client`
// https://github.com/ramosbugs/oauth2-rs/blob/23b952b23e6069525bc7e4c4f2c4924b8d28ce3a/src/reqwest.rs
impl<'c> AsyncHttpClient<'c> for ClientWrapper {
    type Error = OauthError;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + 'c>>;

    fn call(&'c self, req: HttpRequest) -> Self::Future {
        Box::pin(async move {
            // Translate the oauth2::http::Request into a Trillium request
            let mut conn = self
                .0
                .build_conn(req.method(), req.uri().to_string().parse::<Url>()?)
                .with_body(req.body().clone())
                .with_request_headers(Headers::from(req.headers().clone()))
                .await?;
            let status_code: oauth2::http::StatusCode = conn.status().unwrap().try_into()?;
            let body = conn.response_body().read_bytes().await?;

            // Now transform the Trillium response back into an http::Response
            let mut builder = oauth2::http::Response::builder().status(status_code);
            let http_headers: oauth2::http::HeaderMap =
                conn.response_headers().clone().try_into()?;
            builder
                .headers_mut()
                .ok_or_else(|| OauthError::Other("no headers in builder?".into()))?
                .extend(http_headers);
            Ok::<_, OauthError>(builder.body(body)?)
        })
    }
}
