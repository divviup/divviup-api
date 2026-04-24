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

pub const DIVVIUP_API_MEDIA_TYPE: &str = "application/vnd.divviup+json;version=0.1";
#[cfg_attr(not(test), expect(dead_code))] // Used once ReplaceMimeTypesLayer is wired; see TODO in routes.rs.
const APPLICATION_JSON: header::HeaderValue = header::HeaderValue::from_static("application/json");

pub struct ReplaceMimeTypes;

#[trillium::async_trait]
impl Handler for ReplaceMimeTypes {
    async fn run(&self, mut conn: Conn) -> Conn {
        let request_headers = conn.inner_mut().request_headers_mut();
        if let Some(DIVVIUP_API_MEDIA_TYPE) | None = request_headers.get_str(ContentType) {
            request_headers.insert(ContentType, "application/json");
        } else {
            return conn.with_status(UnsupportedMediaType).halt();
        }

        if Some(DIVVIUP_API_MEDIA_TYPE) == request_headers.get_str(Accept) {
            request_headers.insert(Accept, "application/json");
        } else {
            return conn.with_status(NotAcceptable).halt();
        }

        conn
    }

    async fn before_send(&self, conn: Conn) -> Conn {
        conn.with_response_header(ContentType, DIVVIUP_API_MEDIA_TYPE)
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
#[cfg_attr(not(test), expect(dead_code))] // Wired once the Trillium proxy is removed; see TODO in routes.rs.
#[derive(Clone, Debug)]
pub struct ReplaceMimeTypesLayer;

impl<S> Layer<S> for ReplaceMimeTypesLayer {
    type Service = ReplaceMimeTypesService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        ReplaceMimeTypesService { inner }
    }
}

/// Tower [`Service`] produced by [`ReplaceMimeTypesLayer`].
#[cfg_attr(not(test), expect(dead_code))] // Constructed by ReplaceMimeTypesLayer.
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
        match req.headers().get(header::CONTENT_TYPE).map(|v| v.to_str()) {
            Some(Ok(DIVVIUP_API_MEDIA_TYPE)) | None => {
                req.headers_mut()
                    .insert(header::CONTENT_TYPE, APPLICATION_JSON);
            }
            _ => {
                return Box::pin(async { Ok(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()) });
            }
        }

        match req.headers().get(header::ACCEPT).map(|v| v.to_str()) {
            Some(Ok(DIVVIUP_API_MEDIA_TYPE)) => {
                req.headers_mut().insert(header::ACCEPT, APPLICATION_JSON);
            }
            _ => {
                return Box::pin(async { Ok(StatusCode::NOT_ACCEPTABLE.into_response()) });
            }
        }

        // Call the inner service, then set the response Content-Type
        let inner_future = self.inner.call(req);
        Box::pin(async move {
            let mut resp = inner_future.await?;
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(DIVVIUP_API_MEDIA_TYPE),
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
                    .header(header::CONTENT_TYPE, DIVVIUP_API_MEDIA_TYPE)
                    .header(header::ACCEPT, DIVVIUP_API_MEDIA_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            DIVVIUP_API_MEDIA_TYPE
        );
    }

    #[tokio::test]
    async fn accepts_missing_content_type() {
        let resp = test_router()
            .oneshot(
                Request::get("/test")
                    .header(header::ACCEPT, DIVVIUP_API_MEDIA_TYPE)
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
                    .header(header::ACCEPT, DIVVIUP_API_MEDIA_TYPE)
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
                    .header(header::CONTENT_TYPE, DIVVIUP_API_MEDIA_TYPE)
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
                    .header(header::CONTENT_TYPE, DIVVIUP_API_MEDIA_TYPE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
    }
}
