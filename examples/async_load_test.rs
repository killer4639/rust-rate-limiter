use rust_rate_limiter::config::{BasicLoadTestConfig, LoadTestConfig};
use std::time::Instant;
use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let basic_config = BasicLoadTestConfig::default();
    let load_test_config = LoadTestConfig::from_env()?;

    let pool_size: usize = 20;

    let mut clients = Vec::with_capacity(pool_size);
    for _ in 0..pool_size {
        let channel = Channel::from_shared(load_test_config.server_url.clone())
            .unwrap()
            .http2_keep_alive_interval(std::time::Duration::from_secs(30))
            .http2_adaptive_window(true)
            .connect()
            .await?;
        clients.push(RateLimiterClient::new(channel));
    }

    println!("🚀 Starting async load test with {} requests...", basic_config.iteration_count);
    
    let start = Instant::now();
    
    // Spawn all tasks concurrently (don't collect futures first)
    let tasks: Vec<_> = (0..basic_config.iteration_count)
        .map(|request_id| {
            let mut client = clients[(request_id as usize) % pool_size].clone();
            tokio::spawn(async move {
                let request = tonic::Request::new(rate_limiter::HeartBeatRequest {});
                client.heart_beat(request).await
            })
        })
        .collect();
    
    // Await all results concurrently
    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        results.push(task.await);
    }
    
    let elapsed = start.elapsed();
    
    // Calculate stats
    let total = results.len();
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = total - successful;
    
    let duration_secs = elapsed.as_secs_f64();
    let rps = total as f64 / duration_secs;
    
    println!("\n📊 Load Test Results");
    println!("  Total requests: {}", total);
    println!("  Successful: {} ({:.2}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("  Failed: {} ({:.2}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!("  Duration: {:.2}s", duration_secs);
    println!("  Throughput: {:.2} req/s", rps);
    
    Ok(())
}
