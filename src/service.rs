/// Generated service definitions
use tonic::{Request, Response, Status};
use crate::pb::{PingRequest, PingResponse};

#[tonic::async_trait]
pub trait RateLimiter: Send + Sync + 'static {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingResponse>, Status>;
}

pub struct RateLimiterServer<T> {
    inner: T,
}

impl<T: RateLimiter> RateLimiterServer<T> {
    pub fn new(inner: T) -> Self {
        RateLimiterServer { inner }
    }
}

use tonic::body::BoxBody;
use tonic::codegen::Service;
use futures::future::BoxFuture;
use std::task::{Context, Poll};

impl<T: RateLimiter> Service<hyper::Request<BoxBody>> for RateLimiterServer<T> {
    type Response = hyper::Response<BoxBody>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<BoxBody>) -> Self::Future {
        let path = req.uri().path();
        let inner = self.inner.clone_ref();

        Box::pin(async move {
            if path == "/rate_limiter.RateLimiter/Ping" {
                // Decode request
                let body_bytes = hyper::body::to_bytes(req.into_body())
                    .await
                    .unwrap_or_default();
                let request = PingRequest::decode(&body_bytes[..]).unwrap_or_default();
                
                match inner.ping(Request::new(request)).await {
                    Ok(response) => {
                        let response_msg = response.into_inner();
                        let encoded = response_msg.encode_to_vec();
                        let response = hyper::Response::builder()
                            .status(hyper::StatusCode::OK)
                            .body(BoxBody::new(hyper::body::Body::from(encoded)))
                            .unwrap();
                        Ok(response)
                    }
                    Err(status) => {
                        let response = hyper::Response::builder()
                            .status(status.code() as u16)
                            .body(BoxBody::new(hyper::body::Body::empty()))
                            .unwrap();
                        Ok(response)
                    }
                }
            } else {
                let response = hyper::Response::builder()
                    .status(hyper::StatusCode::NOT_FOUND)
                    .body(BoxBody::new(hyper::body::Body::empty()))
                    .unwrap();
                Ok(response)
            }
        })
    }
}
