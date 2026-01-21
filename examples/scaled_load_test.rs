/// Scaled load test - multiple clients, multiple channels, maximum throughput
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tonic::transport::Channel;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration - tune these for max throughput
    // Key insight: more concurrent tasks = more requests in flight simultaneously
    let num_channels: usize = 64;            // More HTTP/2 connections
    let tasks_per_channel: usize = 500;      // Many more concurrent tasks per channel
    let requests_per_task: usize = 100;      // Fewer sequential requests per task
    
    let total_requests = num_channels * tasks_per_channel * requests_per_task;

    println!("🚀 Scaled gRPC Load Test");
    println!("   Channels (connections): {}", num_channels);
    println!("   Tasks per channel: {}", tasks_per_channel);
    println!("   Requests per task: {}", requests_per_task);
    println!("   Total requests: {}", total_requests);
    println!();

    // Create channel pool with HTTP/2 tuning
    let mut channels: Vec<Channel> = Vec::with_capacity(num_channels);
    for _ in 0..num_channels {
        let channel = Channel::from_static("http://127.0.0.1:50051")
            .http2_adaptive_window(true)
            .tcp_nodelay(true)
            .connect()
            .await?;
        channels.push(channel);
    }

    println!("✅ Created {} connections", num_channels);

    // Atomic counters for thread-safe stats
    let success_count = Arc::new(AtomicU64::new(0));
    let failure_count = Arc::new(AtomicU64::new(0));

    let start = Instant::now();

    // Spawn tasks across all channels
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

    println!("✅ Spawned {} tasks", all_tasks.len());

    // Wait for all tasks
    for task in all_tasks {
        let _ = task.await;
    }

    let elapsed = start.elapsed();
    let total_success = success_count.load(Ordering::Relaxed);
    let total_failure = failure_count.load(Ordering::Relaxed);
    let total = total_success + total_failure;
    let rps = total as f64 / elapsed.as_secs_f64();

    println!();
    println!("📊 Scaled Load Test Results");
    println!("  Total requests: {}", total);
    println!("  Successful: {}", total_success);
    println!("  Failed: {}", total_failure);
    println!("  Duration: {:.2}s", elapsed.as_secs_f64());
    println!("  Throughput: {:.0} req/s", rps);

    Ok(())
}
