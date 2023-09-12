use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
    pin::Pin,
    task::{ready, Context, Poll},
};

use super::{AggregatorClient, TaskIds};
use crate::clients::ClientError;
use futures_lite::{stream::Stream, Future};

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

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct TaskIdStream<'a> {
    client: &'a AggregatorClient,
    page: Option<Page>,
    future: Option<BoxFuture<'a, Result<TaskIds, ClientError>>>,
}

impl<'a> TaskIdStream<'a> {
    pub(super) fn new(client: &'a AggregatorClient) -> Self {
        Self {
            client,
            page: None,
            future: None,
        }
    }
}

impl<'a> Debug for TaskIdStream<'a> {
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
