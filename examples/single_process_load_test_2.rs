
/// Single process load test - designed to be run in parallel
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // High capacity config
    let num_channels: usize = 32;
    let tasks_per_channel: usize = 100;
    let requests_per_task: usize = 1000;
    
    let total_requests = num_channels * tasks_per_channel * requests_per_task;

    // Create channel pool
    let mut channels: Vec<Channel> = Vec::with_capacity(num_channels);
    for _ in 0..num_channels {
        let channel = Channel::from_static("http://127.0.0.1:50051")
            .tcp_nodelay(true)
            .http2_adaptive_window(true)
            .connect()
            .await?;
        channels.push(channel);
    }

    let success_count = Arc::new(AtomicU64::new(0));
    let failure_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    let mut all_tasks = Vec::with_capacity(num_channels * tasks_per_channel);

    for channel in channels.into_iter() {
        for _task_id in 0..tasks_per_channel {
            let mut client = RateLimiterClient::new(channel.clone());
            let success = Arc::clone(&success_count);
            let failure = Arc::clone(&failure_count);

            let task = tokio::spawn(async move {
                let mut local_success = 0u64;
                let mut local_failure = 0u64;

                for _req_id in 0..requests_per_task {
                    let request = tonic::Request::new(rate_limiter::HeartBeatRequest {});
                    
                    match client.heart_beat(request).await {
                        Ok(_) => local_success += 1,
                        Err(_) => local_failure += 1,
                    }
                }

                success.fetch_add(local_success, Ordering::Relaxed);
                failure.fetch_add(local_failure, Ordering::Relaxed);
            });

            all_tasks.push(task);
        }
    }

    for task in all_tasks {
        let _ = task.await;
    }

    let elapsed = start.elapsed();
    let total_success = success_count.load(Ordering::Relaxed);
    let total_failure = failure_count.load(Ordering::Relaxed);
    let total = total_success + total_failure;
    let rps = total as f64 / elapsed.as_secs_f64();

    // Output format that can be parsed by aggregator
    println!("requests={} success={} failed={} duration={:.2} rps={:.0}", 
             total, total_success, total_failure, elapsed.as_secs_f64(), rps);
    println!("Throughput: {:.0} req/s", rps);

    Ok(())
}
