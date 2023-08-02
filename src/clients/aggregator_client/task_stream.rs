use std::{
    fmt::{self, Debug, Formatter},
    pin::Pin,
    task::{ready, Context, Poll},
};

use super::{task_id_stream::TaskIdStream, AggregatorClient, TaskResponse};
use crate::clients::ClientError;
use futures_lite::{stream::Stream, Future};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct TaskStream<'a> {
    client: &'a AggregatorClient,
    task_id_stream: TaskIdStream<'a>,
    task_future: Option<BoxFuture<'a, Option<Result<TaskResponse, ClientError>>>>,
}

impl<'a> Debug for TaskStream<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskStream").field("future", &"..").finish()
    }
}

impl<'a> TaskStream<'a> {
    pub(super) fn new(client: &'a AggregatorClient) -> Self {
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
