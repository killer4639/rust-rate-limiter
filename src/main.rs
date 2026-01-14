use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

pub use rate_limiter::rate_limiter_server::{RateLimiter, RateLimiterServer};
pub use rate_limiter::{PingRequest, PingResponse};

const DESCRIPTOR_SET: &[u8] = include_bytes!("../proto/descriptor.bin");

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
    let rate_limiter: RateLimiterService = RateLimiterService::default();

    let reflection = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(DESCRIPTOR_SET)
        .build()?;

    tracing::info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(reflection)
        .add_service(RateLimiterServer::new(rate_limiter))
        .serve(addr)
        .await?;

    Ok(())
}
