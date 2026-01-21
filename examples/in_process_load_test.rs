/// In-process load test - tests the actual rate limiter capacity without gRPC overhead
use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;
use std::time::Duration;

struct TokenBucket {
    tokens: i32,
    last_refill: Instant,
}

struct RateLimiter {
    state: Arc<DashMap<String, TokenBucket>>,
    tokens_per_window: i32,
    window_duration: Duration,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            state: Arc::new(DashMap::new()),
            tokens_per_window: 10,
            window_duration: Duration::from_secs(60),
        }
    }

    fn check_rate_limit(&self, id: &str, tokens_requested: i32) -> bool {
        let now = Instant::now();

        let mut bucket = self
            .state
            .entry(id.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.tokens_per_window,
                last_refill: now,
            });

        let elapsed = now.duration_since(bucket.last_refill);
        if elapsed >= self.window_duration {
            bucket.tokens = self.tokens_per_window;
            bucket.last_refill = now;
        }

        if bucket.tokens >= tokens_requested {
            bucket.tokens -= tokens_requested;
            true
        } else {
            false
        }
    }
}

#[tokio::main]
async fn main() {
    let iteration_count: u64 = 1_000_000;
    let num_tasks: u64 = 100;
    let requests_per_task = iteration_count / num_tasks;

    let rate_limiter = Arc::new(RateLimiter::new());

    println!("🚀 In-process load test: {} requests across {} tasks", iteration_count, num_tasks);
    println!("   (No gRPC, no network - pure rate limiter performance)\n");

    let start = Instant::now();

    let tasks: Vec<_> = (0..num_tasks)
        .map(|task_id| {
            let rl = Arc::clone(&rate_limiter);
            tokio::spawn(async move {
                let mut allowed = 0u64;
                let mut denied = 0u64;
                for i in 0..requests_per_task {
                    let user_id = format!("user-{}", (task_id * requests_per_task + i) % 1000);
                    if rl.check_rate_limit(&user_id, 1) {
                        allowed += 1;
                    } else {
                        denied += 1;
                    }
                }
                (allowed, denied)
            })
        })
        .collect();

    let mut total_allowed = 0u64;
    let mut total_denied = 0u64;
    for task in tasks {
        let (allowed, denied) = task.await.unwrap();
        total_allowed += allowed;
        total_denied += denied;
    }

    let elapsed = start.elapsed();
    let total = total_allowed + total_denied;
    let rps = total as f64 / elapsed.as_secs_f64();

    println!("📊 In-Process Load Test Results");
    println!("  Total requests: {}", total);
    println!("  Allowed: {} ({:.2}%)", total_allowed, (total_allowed as f64 / total as f64) * 100.0);
    println!("  Denied: {} ({:.2}%)", total_denied, (total_denied as f64 / total as f64) * 100.0);
    println!("  Duration: {:.2}s", elapsed.as_secs_f64());
    println!("  Throughput: {:.2} req/s", rps);
    println!("\n✅ This is your rate limiter's TRUE capacity (without network overhead)");
}
