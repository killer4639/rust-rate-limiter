use std::time::Instant;

async fn dummy_work() -> u64 {
    // Simulate minimal async work
    1
}

#[tokio::main]
async fn main() {
    let iteration_count: u64 = 100_000;

    println!("🚀 Starting PURE async test with {} tasks...", iteration_count);

    let start = Instant::now();

    let tasks: Vec<_> = (0..iteration_count)
        .map(|_| tokio::spawn(async move { dummy_work().await }))
        .collect();

    let mut sum: u64 = 0;
    for task in tasks {
        sum += task.await.unwrap();
    }

    let elapsed = start.elapsed();
    let rps = iteration_count as f64 / elapsed.as_secs_f64();

    println!("\n📊 Pure Async Results");
    println!("  Tasks completed: {}", sum);
    println!("  Duration: {:.2}s", elapsed.as_secs_f64());
    println!("  Throughput: {:.2} tasks/sec", rps);
}
