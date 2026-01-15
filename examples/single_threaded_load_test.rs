use rust_rate_limiter::config::{BasicLoadTestConfig, ServerConfig};
use std::time::Instant;
use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::HeartBeatRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BasicLoadTestConfig::default();
    let server_url = ServerConfig::from_env().url();
    println!(
        "Starting load test iteration with count: {}",
        config.iteration_count
    );

    let start_time = Instant::now();

    match Channel::from_shared(server_url).and_then(|ch| {
        let ch = ch.connect_lazy();
        Ok(RateLimiterClient::new(ch))
    }) {
        Ok(mut client) => {
            for request_id in 0..config.iteration_count {
                let request = tonic::Request::new(HeartBeatRequest {});
                let result = client
                    .heart_beat(request)
                    .await
                    .map(|_| (String::from("hb"), request_id, true));
            }
        }
        Err(e) => {
            eprintln!("System failed to connect: {}", e);
        }
    }

    let end_time = Instant::now();
    println!(
        "Load test completed in {} sec",
        (end_time - start_time).as_secs_f64()
    );
    let throughput = config.iteration_count as f64 / (end_time - start_time).as_secs_f64();
    println!(
        "Load test completed. Throughput: {:.2} requests/sec",
        throughput
    );
    Ok(())
}
