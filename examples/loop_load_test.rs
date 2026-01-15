use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Starting loop test"
    );

    let start_time = Instant::now();
    
    // Note: You'll also need to update the throughput calculation below to use milliseconds:
    // let elapsed_ms = (end_time - start_time).as_millis();
    // let throughput = config.iteration_count as f64 / (elapsed_ms as f64 / 1000.0);

    let mut counter: u64 = 0;

    for _ in 0..100000000 {
        counter+=1;
    }

    let end_time = Instant::now();
    println!(
        "Load test completed in {} sec",
        (end_time - start_time).as_secs_f64()
    );
    let throughput = counter as f64 / (end_time - start_time).as_secs_f64();
    println!(
        "Load test completed. Throughput: {:.2} requests/sec",
        throughput
    );
    Ok(())
}
