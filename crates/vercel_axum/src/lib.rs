use axum::response::IntoResponse;
use base64::prelude::*;
use http_body_util::BodyExt;
use std::{future::Future, pin::Pin};
use tower::Layer;
use tower_service::Service;

use vercel_runtime::request::{Event, VercelRequest};
use vercel_runtime::response::EventResponse;

#[derive(Clone, Copy)]
pub struct VercelLayer;

impl<S> Layer<S> for VercelLayer {
    type Service = VercelService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VercelService { inner }
    }
}

pub struct VercelService<S> {
    inner: S,
}

impl<S> Service<Event<'_>> for VercelService<S>
where
    S: Service<axum::http::Request<axum::body::Body>>,
    S::Response: axum::response::IntoResponse + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = EventResponse;
    type Error = vercel_runtime::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, event: Event) -> Self::Future {
        let (event, _context) = event.into_parts();
        let request = serde_json::from_str::<VercelRequest>(&event.body).unwrap_or_default();

        let mut builder = axum::http::request::Builder::new()
            .method(request.method)
            .uri(format!("https://{}{}", request.host, request.path));
        for (key, value) in request.headers {
            if let Some(k) = key {
                builder = builder.header(k, value);
            }
        }

        let request: axum::http::Request<axum::body::Body> = match (request.body, request.encoding)
        {
            (Some(b), Some(encoding)) if encoding == "base64" => {
                let engine = base64::prelude::BASE64_STANDARD;
                let body = axum::body::Body::from(engine.decode(b.as_ref()).unwrap_or_default());
                builder.body(body).unwrap_or_default()
            }
            (Some(b), _) => builder.body(axum::body::Body::from(b)).unwrap_or_default(),
            (None, _) => builder.body(axum::body::Body::default()).unwrap_or_default(),
        };

        let fut = self.inner.call(request);
        let fut = async move {
            let resp = fut.await?;
            let (parts, body) = resp.into_response().into_parts();
            let bytes = body.into_data_stream().collect().await?.to_bytes();
            let bytes: &[u8] = &bytes;
            let body = std::str::from_utf8(bytes).unwrap_or_default();
            let body: Option<vercel_runtime::Body> = match body {
                "" => None,
                _ => Some(body.into()),
            };
            Ok(EventResponse {
                status_code: parts.status.as_u16(),
                body,
                headers: parts.headers,
                encoding: None,
            })
        };

        Box::pin(fut)
    }
}
