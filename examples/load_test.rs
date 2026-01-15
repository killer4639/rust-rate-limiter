use tonic::transport::Channel;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::task::JoinHandle;
use rand::Rng;

// Import config from the main crate
use rust_rate_limiter::config::LoadTestConfig;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::RateLimitRequest;

#[derive(Clone, Debug)]
struct RequestStats {
    id: String,
    success: bool,
    latency_ms: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = LoadTestConfig::from_env()?;
    config.print_summary();

    let stats = Arc::new(Mutex::new(Vec::new()));
    let start = Instant::now();

    // Spawn tasks
    let mut handles: Vec<JoinHandle<()>> = vec![];
    
    for thread_id in 0..config.num_threads {
        let stats = Arc::clone(&stats);
        let server_url = config.server_url.clone();
        let requests_per_thread = config.requests_per_thread;
        
        let handle = tokio::spawn(async move {
            match Channel::from_shared(server_url)
                .and_then(|ch| {
                    let ch = ch.connect_lazy();
                    Ok(RateLimiterClient::new(ch))
                }) {
                Ok(mut client) => {
                    for request_id in 0..requests_per_thread {
                        // Generate random user ID inside the loop (thread-safe)
                        let random_id = (thread_id * requests_per_thread + request_id) % 1000 + 1;
                        let user_id = format!("user-{}", random_id);
                        let tokens = (request_id % 5 + 1) as i32; // Vary tokens 1-5
                        
                        let request_start = Instant::now();
                        let request = tonic::Request::new(RateLimitRequest {
                            id: user_id.clone(),
                            tokens_requested: tokens,
                        });
                        
                        match client.ping(request).await {
                            Ok(response) => {
                                let latency = request_start.elapsed().as_secs_f64() * 1000.0;
                                let success = response.into_inner().status == "success";
                                
                                let stat = RequestStats {
                                    id: format!("{}-{}", user_id, request_id),
                                    success,
                                    latency_ms: latency,
                                };
                                
                                stats.lock().unwrap().push(stat);
                            }
                            Err(status) => {
                                let latency = request_start.elapsed().as_secs_f64() * 1000.0;
                                
                                // Rate limited errors are expected
                                let success = status.code() == tonic::Code::ResourceExhausted;
                                
                                let stat = RequestStats {
                                    id: format!("{}-{}", user_id, request_id),
                                    success,
                                    latency_ms: latency,
                                };
                                
                                stats.lock().unwrap().push(stat);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Thread {} failed to connect: {}", thread_id, e);
                }
            }
        });
        
        handles.push(handle);
    }

    // Wait for all tasks to complete
    println!("⏳ Running load test...");
    for handle in handles {
        handle.await;
    }

    let elapsed = start.elapsed();
    let stats_lock = stats.lock().unwrap();
    let all_stats = stats_lock.clone();

    // Print statistics
    println!("\n📊 Load Test Results");
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  Total requests: {}", all_stats.len());
    
    let successful = all_stats.iter().filter(|s| s.success).count();
    let failed = all_stats.len() - successful;
    
    println!("  Successful: {} ({:.2}%)", successful, (successful as f64 / all_stats.len() as f64) * 100.0);
    println!("  Failed/Rate Limited: {} ({:.2}%)", failed, (failed as f64 / all_stats.len() as f64) * 100.0);
    
    // Latency statistics
    let mut latencies: Vec<f64> = all_stats.iter().map(|s| s.latency_ms).collect();
    let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[(latencies.len() * 99) / 100];
    
    println!();
    println!("⏱️  Latency (ms)");
    println!("  Min: {:.3}", min_latency);
    println!("  Max: {:.3}", max_latency);
    println!("  Avg: {:.3}", avg_latency);
    println!("  P50: {:.3}", p50);
    println!("  P99: {:.3}", p99);
    
    let rps = all_stats.len() as f64 / elapsed.as_secs_f64();
    println!();
    println!("🚀 Throughput");
    println!("  Requests/sec: {:.2}", rps);
    
    Ok(())
}
