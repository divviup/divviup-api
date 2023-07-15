use crate::{
    clients::{ClientConnExt, ClientError},
    Config,
};
use email_address::EmailAddress;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use trillium::{async_trait, Conn, KnownHeaderName};
use trillium_api::FromConn;
use trillium_client::Client;
use url::Url;

#[derive(Debug, Clone)]
pub struct PostmarkClient {
    token: String,
    client: Client,
    email: EmailAddress,
    base_url: Url,
}

#[async_trait]
impl FromConn for PostmarkClient {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

impl PostmarkClient {
    pub fn new(config: &Config) -> Self {
        Self {
            token: config.postmark_token.clone(),
            client: config.client.clone(),
            email: config.email_address.clone(),
            base_url: config.postmark_url.clone(),
        }
    }

    pub async fn send_email(&self, email: Email) -> Result<Value, ClientError> {
        let mut email = serde_json::to_value(&email)?;
        email
            .as_object_mut()
            .unwrap()
            .insert("From".into(), self.email.to_string().into());
        self.post("/email", &email).await
    }

    pub async fn send_email_template(
        &self,
        to: &str,
        template_name: &str,
        model: &impl Serialize,
        message_id: Option<String>,
    ) -> Result<Value, ClientError> {
        let headers = match message_id {
            Some(m) => json!([{
                "Name": "Message-ID",
                "Value": format!("<{m}@{}>", self.email.domain())
            }]),
            None => json!([]),
        };

        self.post(
            "/email/withTemplate",
            &json!({
                "To": to,
                "From": self.email,
                "TemplateAlias": template_name,
                "TemplateModel": model,
                "Headers": headers
            }),
        )
        .await
    }

    // private below here

    async fn post<T>(&self, path: &str, json: &impl Serialize) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        self.client
            .post(self.base_url.join(path).unwrap())
            .with_header("X-Postmark-Server-Token", self.token.clone())
            .with_header(KnownHeaderName::Accept, "application/json")
            .with_json_body(json)?
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}
