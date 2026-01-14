use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::PingRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    
    let mut client = RateLimiterClient::new(channel);
    
    let request = tonic::Request::new(PingRequest {
        message: "Hello from gRPC client!".to_string(),
    });
    
    let response = client.ping(request).await?;
    
    println!("Response: {:#?}", response.into_inner());
    
    Ok(())
}
