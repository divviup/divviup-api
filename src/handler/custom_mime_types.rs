use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use axum::response::IntoResponse;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use trillium::{
    Conn, Handler,
    KnownHeaderName::{Accept, ContentType},
    Status::{NotAcceptable, UnsupportedMediaType},
};

pub const CONTENT_TYPE: &str = "application/vnd.divviup+json;version=0.1";

pub struct ReplaceMimeTypes;

#[trillium::async_trait]
impl Handler for ReplaceMimeTypes {
    async fn run(&self, mut conn: Conn) -> Conn {
        let request_headers = conn.inner_mut().request_headers_mut();
        if let Some(CONTENT_TYPE) | None = request_headers.get_str(ContentType) {
            request_headers.insert(ContentType, "application/json");
        } else {
            return conn.with_status(UnsupportedMediaType).halt();
        }

        if Some(CONTENT_TYPE) == request_headers.get_str(Accept) {
            request_headers.insert(Accept, "application/json");
        } else {
            return conn.with_status(NotAcceptable).halt();
        }

        conn
    }

    async fn before_send(&self, conn: Conn) -> Conn {
        conn.with_response_header(ContentType, CONTENT_TYPE)
    }
}

// ---------------------------------------------------------------------------
// Axum / Tower equivalent of ReplaceMimeTypes
// ---------------------------------------------------------------------------

/// Tower [`Layer`] that applies [`ReplaceMimeTypesService`] to an inner service.
///
/// This replicates the Trillium [`ReplaceMimeTypes`] handler for Axum routes:
/// requests with the custom content type (or no content type) have their headers
/// normalized to `application/json`; responses get the custom content type set.
#[cfg_attr(not(test), expect(dead_code))] // Wired onto the API sub-router in Part 7.
#[derive(Clone, Debug)]
pub struct ReplaceMimeTypesLayer;

impl<S> Layer<S> for ReplaceMimeTypesLayer {
    type Service = ReplaceMimeTypesService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        ReplaceMimeTypesService { inner }
    }
}

/// Tower [`Service`] produced by [`ReplaceMimeTypesLayer`].
#[cfg_attr(not(test), expect(dead_code))] // Constructed by ReplaceMimeTypesLayer; wired in Part 7.
#[derive(Clone, Debug)]
pub struct ReplaceMimeTypesService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for ReplaceMimeTypesService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // --- Request-side: Content-Type ---
        let ct = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        match ct.as_deref() {
            Some(CONTENT_TYPE) | None => {
                req.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
            }
            _ => {
                return Box::pin(async { Ok(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()) });
            }
        }

        // --- Request-side: Accept ---
        let accept = req
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok());
        if accept == Some(CONTENT_TYPE) {
            req.headers_mut().insert(
                header::ACCEPT,
                header::HeaderValue::from_static("application/json"),
            );
        } else {
            return Box::pin(async { Ok(StatusCode::NOT_ACCEPTABLE.into_response()) });
        }

        // --- Call the inner service, then set response Content-Type ---
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let mut resp = inner.call(req).await?;
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(CONTENT_TYPE),
            );
            Ok(resp)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(ReplaceMimeTypesLayer)
    }

    #[tokio::test]
    async fn accepts_custom_content_type() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::CONTENT_TYPE, CONTENT_TYPE)
                    .header(header::ACCEPT, CONTENT_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            CONTENT_TYPE
        );
    }

    #[tokio::test]
    async fn accepts_missing_content_type() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::ACCEPT, CONTENT_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_wrong_content_type() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .header(header::ACCEPT, CONTENT_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn rejects_wrong_accept() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::CONTENT_TYPE, CONTENT_TYPE)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[tokio::test]
    async fn rejects_missing_accept() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::CONTENT_TYPE, CONTENT_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
    }
}
