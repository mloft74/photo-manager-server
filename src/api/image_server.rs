use std::{error::Error, str::from_utf8};

use axum::{body::Bytes, extract::Request, response::Response, Router};
use futures::FutureExt;
use http_body::Body;
use serde_querystring::UrlEncodedQS;
use tower::Service;
use tower_http::services::ServeDir;

use crate::api::IMAGES_DIR;

pub fn create_image_server_router() -> Router {
    Router::new().nest_service("/image", ServeDir::new(IMAGES_DIR))
}

#[derive(Debug, Clone)]
struct Resize<S> {
    inner: S,
}

impl<S, ReqBody, SResBody> Service<Request<ReqBody>> for Resize<S>
where
    S: Service<Request<ReqBody>, Response = Response<SResBody>>,
    S::Future: Send + 'static,
    SResBody: Body<Data = Bytes> + Send + 'static,
    SResBody::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let query = req.uri().query();
        if let Some(query) = query {
            let parsed = UrlEncodedQS::parse(query.as_bytes());
            let width = parsed
                .value(b"width")
                .flatten()
                .and_then(|v| parse_dimension_from_query_val(&v));
            let height = parsed
                .value(b"height")
                .flatten()
                .and_then(|v| parse_dimension_from_query_val(&v));

            let dim = width.zip(height);

            if let Some((width, height)) = dim {
                self.inner.call(req)
            } else {
                self.inner.call(req)
            }
        } else {
            self.inner.call(req)
        }
    }
}

fn parse_dimension_from_query_val(val: &[u8]) -> Option<u32> {
    from_utf8(val).ok().and_then(|val| val.parse().ok())
}
