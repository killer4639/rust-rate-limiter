use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::RateLimitRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    
    let mut client = RateLimiterClient::new(channel);
    
    let request = tonic::Request::new(RateLimitRequest {
        id: "1234".to_string(),
        tokens_requested: 5,
    });
    
    let response = client.check_rate_limit(request).await?;
    
    println!("Response: {:#?}", response.into_inner());
    
    Ok(())
}
