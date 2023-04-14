use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use trillium::KnownHeaderName;
use trillium_api::FromConn;
use url::Url;

use crate::{
    client::{expect_ok, ClientError},
    ApiConfig,
};
type ClientConnector = trillium_rustls::RustlsConnector<trillium_tokio::TcpConnector>;
type Client = trillium_client::Client<ClientConnector>;

#[derive(Debug, Clone)]
pub struct PostmarkClient {
    token: String,
    client: Client,
    email: String,
    base_url: Url,
}

#[trillium::async_trait]
impl FromConn for PostmarkClient {
    async fn from_conn(conn: &mut trillium::Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

impl PostmarkClient {
    pub fn new(config: &ApiConfig) -> Self {
        Self {
            token: config.postmark_token.clone(),
            client: Client::new().with_default_pool(),
            email: config.email_address.clone(),
            base_url: Url::parse("https://api.postmarkapp.com").unwrap(),
        }
    }

    pub fn with_http_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    async fn post<T>(&self, path: &str, json: &impl Serialize) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let mut conn = self
            .client
            .post(self.base_url.join(path).unwrap())
            .with_header("X-Postmark-Server-Token", self.token.clone())
            .with_header(KnownHeaderName::Accept, "application/json")
            .with_json_body(json)?
            .await?;
        expect_ok(&mut conn).await?;
        Ok(conn.response_json().await?)
    }

    pub async fn send_email(&self, email: Email) -> Result<Value, ClientError> {
        let mut email = serde_json::to_value(&email)?;
        email
            .as_object_mut()
            .unwrap()
            .insert("From".into(), self.email.clone().into());
        self.post("/email", &email).await
    }

    pub async fn send_email_template(
        &self,
        to: &str,
        template_name: &str,
        model: &impl Serialize,
    ) -> Result<Value, ClientError> {
        self.post(
            "/email/withTemplate",
            &json!({
                "To": to,
                "From": self.email,
                "TemplateAlias": template_name,
                "TemplateModel": model
            }),
        )
        .await
    }

    pub fn set_http_client(&mut self, client: Client) {
        self.client = client;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}
