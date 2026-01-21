use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;

mod config;
mod rate_limiter_service;

use config::ServerConfig;
use rate_limiter_service::RateLimiterService;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_server::RateLimiterServer;

const DESCRIPTOR_SET: &[u8] = include_bytes!("../proto/descriptor.bin");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_config = ServerConfig::from_env();
    let addr = server_config.socket_addr().parse()?;
    let rate_limiter: RateLimiterService = RateLimiterService::default();

    let reflection = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(DESCRIPTOR_SET)
        .build()?;

    println!("🚀 High-performance gRPC server listening on {}", addr);

    Server::builder()
        .concurrency_limit_per_connection(5000)
        .tcp_nodelay(true)
        .add_service(reflection)
        .add_service(RateLimiterServer::new(rate_limiter))
        .serve(addr)
        .await?;

    Ok(())
}
