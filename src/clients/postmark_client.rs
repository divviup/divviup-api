use crate::{
    clients::{ClientConnExt, ClientError},
    ApiConfig,
};
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
    email: String,
    base_url: Url,
}

#[async_trait]
impl FromConn for PostmarkClient {
    async fn from_conn(conn: &mut Conn) -> Option<Self> {
        conn.state().cloned()
    }
}

impl PostmarkClient {
    pub fn new(config: &ApiConfig) -> Self {
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
