use std::{
    collections::VecDeque,
    fmt::{self, Formatter},
    pin::Pin,
    task::{ready, Context, Poll},
};

use crate::{
    clients::{ClientConnExt, ClientError},
    entity::{task::ProvisionableTask, Aggregator},
    handler::Error,
};
use futures_lite::{stream::Stream, Future, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use trillium::{HeaderValue, KnownHeaderName, Method};
use trillium_client::{Client, Conn};
use url::Url;
pub mod api_types;
pub use api_types::{AggregatorApiConfig, TaskCreate, TaskIds, TaskMetrics, TaskResponse};

const CONTENT_TYPE: &str = "application/vnd.janus.aggregator+json;version=0.1";

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    client: Client,
    auth_header: HeaderValue,
    aggregator: Aggregator,
    base_url: Url,
}

impl AsRef<Client> for AggregatorClient {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}

impl AggregatorClient {
    pub fn new(client: Client, aggregator: Aggregator, bearer_token: &str) -> Self {
        let mut base_url: Url = aggregator.api_url.clone().into();
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        Self {
            client,
            auth_header: format!("Bearer {bearer_token}").into(),
            aggregator,
            base_url,
        }
    }

    pub async fn get_config(
        client: Client,
        base_url: Url,
        token: &str,
    ) -> Result<AggregatorApiConfig, ClientError> {
        client
            .get(base_url)
            .with_header(KnownHeaderName::Authorization, format!("Bearer {token}"))
            .with_header(KnownHeaderName::Accept, CONTENT_TYPE)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_task_id_page(&self, page: Option<&str>) -> Result<TaskIds, ClientError> {
        let path = if let Some(pagination_token) = page {
            format!("task_ids?pagination_token={pagination_token}")
        } else {
            "task_ids".into()
        };
        self.get(&path).await
    }

    pub async fn get_task_ids(&self) -> Result<Vec<String>, ClientError> {
        self.task_id_stream().try_collect().await
    }

    pub async fn get_task(&self, task_id: &str) -> Result<TaskResponse, ClientError> {
        self.get(&format!("tasks/{task_id}")).await
    }

    pub async fn get_task_metrics(&self, task_id: &str) -> Result<TaskMetrics, ClientError> {
        self.get(&format!("tasks/{task_id}/metrics")).await
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), ClientError> {
        self.delete(&format!("tasks/{task_id}")).await
    }

    pub async fn create_task(&self, task: &ProvisionableTask) -> Result<TaskResponse, Error> {
        let task_create = TaskCreate::build(&self.aggregator, task)?;
        self.post("tasks", &task_create).await.map_err(Into::into)
    }

    // private below here

    fn url(&self, path: &str) -> Url {
        self.base_url.join(path).unwrap()
    }

    fn conn(&self, method: Method, path: &str) -> Conn {
        self.client
            .build_conn(method, self.url(path))
            .with_header(KnownHeaderName::Authorization, self.auth_header.clone())
            .with_header(KnownHeaderName::Accept, CONTENT_TYPE)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        self.conn(Method::Get, path)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, ClientError> {
        self.conn(Method::Post, path)
            .with_json_body(body)?
            .with_header(KnownHeaderName::ContentType, CONTENT_TYPE)
            .success_or_client_error()
            .await?
            .response_json()
            .await
            .map_err(ClientError::from)
    }

    async fn delete(&self, path: &str) -> Result<(), ClientError> {
        let _ = self
            .conn(Method::Delete, path)
            .success_or_client_error()
            .await?;
        Ok(())
    }

    pub fn task_id_stream(&self) -> TaskIdStream<'_> {
        TaskIdStream::new(self)
    }

    pub fn task_stream(&self) -> TaskStream<'_> {
        TaskStream::new(self)
    }
}

#[derive(Clone, Debug)]
struct Page {
    task_ids: VecDeque<String>,
    pagination_token: Option<String>,
}

impl From<TaskIds> for Page {
    fn from(
        TaskIds {
            task_ids,
            pagination_token,
        }: TaskIds,
    ) -> Self {
        Page {
            task_ids: task_ids.into_iter().map(|t| t.to_string()).collect(),
            pagination_token,
        }
    }
}

pub struct TaskIdStream<'a> {
    client: &'a AggregatorClient,
    page: Option<Page>,
    future: Option<Pin<Box<dyn Future<Output = Result<TaskIds, ClientError>> + Send + 'a>>>,
}

impl<'a> TaskIdStream<'a> {
    fn new(client: &'a AggregatorClient) -> Self {
        Self {
            client,
            page: None,
            future: None,
        }
    }
}

impl<'a> fmt::Debug for TaskIdStream<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskIdStream")
            .field("client", &self.client)
            .field("current_page", &self.page)
            .field("current_future", &"..")
            .finish()
    }
}

impl Stream for TaskIdStream<'_> {
    type Item = Result<String, ClientError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            client,
            ref mut page,
            ref mut future,
        } = *self;

        loop {
            if let Some(page) = page {
                if let Some(task_id) = page.task_ids.pop_front() {
                    return Poll::Ready(Some(Ok(task_id)));
                }

                if page.pagination_token.is_none() {
                    return Poll::Ready(None);
                }
            }

            if let Some(fut) = future {
                *page = Some(ready!(Pin::new(&mut *fut).poll(cx))?.into());
                *future = None;
            } else {
                let pagination_token = page.as_ref().and_then(|page| page.pagination_token.clone());

                *future = Some(Box::pin(async move {
                    client.get_task_id_page(pagination_token.as_deref()).await
                }));
            };
        }
    }
}

pub struct TaskStream<'a> {
    client: &'a AggregatorClient,
    task_id_stream: TaskIdStream<'a>,
    task_future: Option<
        Pin<Box<dyn Future<Output = Option<Result<TaskResponse, ClientError>>> + Send + 'a>>,
    >,
}

impl<'a> fmt::Debug for TaskStream<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskStream").field("future", &"..").finish()
    }
}

impl<'a> TaskStream<'a> {
    fn new(client: &'a AggregatorClient) -> Self {
        Self {
            task_id_stream: client.task_id_stream(),
            client,
            task_future: None,
        }
    }
}

impl Stream for TaskStream<'_> {
    type Item = Result<TaskResponse, ClientError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            client,
            ref mut task_id_stream,
            ref mut task_future,
        } = *self;

        loop {
            if let Some(future) = task_future {
                let res = ready!(Pin::new(&mut *future).poll(cx));
                *task_future = None;
                return Poll::Ready(res);
            }

            *task_future = match ready!(Pin::new(&mut *task_id_stream).poll_next(cx)) {
                Some(Ok(task_id)) => Some(Box::pin(async move {
                    let task_id = task_id;
                    Some(client.get_task(&task_id).await)
                })),
                None => return Poll::Ready(None),
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
            };
        }
    }
}
