use tonic::{transport::Server, Request, Response, Status};

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_server::{RateLimiter, RateLimiterServer};
use rate_limiter::{PingRequest, PingResponse};

#[derive(Default)]
pub struct RateLimiterService {}

#[tonic::async_trait]
impl RateLimiter for RateLimiterService {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Received ping request: {:?}", req.message);

        let reply = PingResponse {
            status: "success".to_string(),
            message: format!("Pong: {}", req.message),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:50051".parse()?;
    let rate_limiter = RateLimiterService::default();

    tracing::info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(RateLimiterServer::new(rate_limiter))
        .serve(addr)
        .await?;

    Ok(())
}
