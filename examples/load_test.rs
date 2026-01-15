use tonic::transport::Channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use std::env;

// Import config from the main crate
use rust_rate_limiter::config::LoadTestConfig;

pub mod rate_limiter {
    tonic::include_proto!("rate_limiter");
}

use rate_limiter::rate_limiter_client::RateLimiterClient;
use rate_limiter::{RateLimitRequest, HeartBeatRequest};

#[derive(Clone, Debug)]
struct RequestStats {
    id: String,
    success: bool,
    latency_ms: f64,
}

#[derive(Clone, Copy, Debug)]
enum EndpointKind {
    CheckRateLimit,
    HeartBeat,
}

async fn run_load_test(
    endpoint: EndpointKind,
    config: &LoadTestConfig,
) -> Result<Vec<RequestStats>, Box<dyn std::error::Error>> {
    let stats = Arc::new(Mutex::new(Vec::new()));

    // Spawn tasks
    let mut handles: Vec<JoinHandle<()>> = vec![];

    for thread_id in 0..config.num_threads {
        let stats = Arc::clone(&stats);
        let server_url = config.server_url.clone();
        let requests_per_thread = config.requests_per_thread;

        let handle = tokio::spawn(async move {
            match Channel::from_shared(server_url).and_then(|ch| {
                let ch = ch.connect_lazy();
                Ok(RateLimiterClient::new(ch))
            }) {
                Ok(mut client) => {
                    for request_id in 0..requests_per_thread {
                        let request_start = Instant::now();

                        let result = match endpoint {
                            EndpointKind::CheckRateLimit => {
                                // Deterministic user id distribution
                                let random_id = (thread_id * requests_per_thread + request_id) % 1000 + 1;
                                let user_id = format!("user-{}", random_id);
                                let tokens = (request_id % 5 + 1) as i32; // Vary tokens 1-5

                                let request = tonic::Request::new(RateLimitRequest {
                                    id: user_id.clone(),
                                    tokens_requested: tokens,
                                });

                                client.check_rate_limit(request).await.map(|resp| {
                                    (user_id, request_id, resp.into_inner().status == "success")
                                })
                            }
                            EndpointKind::HeartBeat => {
                                let request = tonic::Request::new(HeartBeatRequest {});
                                client.heart_beat(request).await.map(|_| {
                                    (String::from("hb"), request_id, true)
                                })
                            }
                        };

                        match result {
                            Ok((user_id, request_id, success)) => {
                                let latency = request_start.elapsed().as_secs_f64() * 1000.0;
                                let stat = RequestStats {
                                    id: format!("{}-{}", user_id, request_id),
                                    success,
                                    latency_ms: latency,
                                };
                                stats.lock().unwrap().push(stat);
                            }
                            Err(status) => {
                                let latency = request_start.elapsed().as_secs_f64() * 1000.0;
                                let success = match endpoint {
                                    EndpointKind::CheckRateLimit => status.code() == tonic::Code::ResourceExhausted,
                                    EndpointKind::HeartBeat => false,
                                };
                                let stat = RequestStats {
                                    id: format!("err-{}", request_id),
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

    for handle in handles {
        let _ = handle.await;
    }

    Ok(Arc::try_unwrap(stats).unwrap().into_inner().unwrap())
}

fn print_results(name: &str, stats: &[RequestStats], elapsed: Duration) {
    println!("\n📊 {} Results", name);
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  Total requests: {}", stats.len());

    let successful = stats.iter().filter(|s| s.success).count();
    let failed = stats.len() - successful;

    println!(
        "  Successful: {} ({:.2}%)",
        successful,
        (successful as f64 / stats.len() as f64) * 100.0
    );
    println!(
        "  Failed: {} ({:.2}%)",
        failed,
        (failed as f64 / stats.len() as f64) * 100.0
    );

    let mut latencies: Vec<f64> = stats.iter().map(|s| s.latency_ms).collect();
    let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[(latencies.len() * 99) / 100];

    println!("\n⏱️  {} Latency (ms)", name);
    println!("  Min: {:.3}", min_latency);
    println!("  Max: {:.3}", max_latency);
    println!("  Avg: {:.3}", avg_latency);
    println!("  P50: {:.3}", p50);
    println!("  P99: {:.3}", p99);

    let rps = stats.len() as f64 / elapsed.as_secs_f64();
    println!("\n🚀 {} Throughput", name);
    println!("  Requests/sec: {:.2}", rps);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = LoadTestConfig::from_env()?;
    config.print_summary();

    // Parse CLI arg: rate | heartbeat | both
    let arg = env::args().nth(1).unwrap_or_else(|| "rate".to_string());
    let run_both = arg.eq_ignore_ascii_case("both");

    if run_both || arg.eq_ignore_ascii_case("rate") || arg.eq_ignore_ascii_case("check") {
        println!("⏳ Running load test: CheckRateLimit...");
        let start = Instant::now();
        let stats = run_load_test(EndpointKind::CheckRateLimit, &config).await?;
        let elapsed = start.elapsed();
        print_results("CheckRateLimit", &stats, elapsed);
    }

    if run_both || arg.eq_ignore_ascii_case("heartbeat") || arg.eq_ignore_ascii_case("hb") {
        println!("\n⏳ Running load test: HeartBeat...");
        let start = Instant::now();
        let stats = run_load_test(EndpointKind::HeartBeat, &config).await?;
        let elapsed = start.elapsed();
        print_results("HeartBeat", &stats, elapsed);
    }

    if !run_both
        && !arg.eq_ignore_ascii_case("rate")
        && !arg.eq_ignore_ascii_case("check")
        && !arg.eq_ignore_ascii_case("heartbeat")
        && !arg.eq_ignore_ascii_case("hb")
    {
        eprintln!("Unknown option: {}", arg);
        eprintln!("Usage: cargo run --example load_test -- [rate|heartbeat|both]");
    }

    Ok(())
}
