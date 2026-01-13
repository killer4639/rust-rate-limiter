use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::PingRequest;

#[tokio::test]
async fn test_ping_request() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    
    let mut client = RateLimiterClient::new(channel);
    
    let request = tonic::Request::new(PingRequest {
        message: "Hello from gRPC test!".to_string(),
    });
    
    let response = client.ping(request).await?;
    let response_msg = response.into_inner();
    
    assert_eq!(response_msg.status, "success");
    assert!(response_msg.message.contains("Pong"));
    
    Ok(())
}
