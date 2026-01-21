/// Multi-process load test runner
/// Spawns multiple load test processes and aggregates results
use std::process::Command;
use std::time::Instant;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_processes = 8;  // Number of parallel load test processes
    
    println!("🚀 Multi-Process Load Test");
    println!("   Spawning {} parallel load test processes", num_processes);
    println!("   Server: single process on 127.0.0.1:50051");
    println!();

    let start = Instant::now();
    
    // Spawn processes
    let mut handles = Vec::new();
    for i in 0..num_processes {
        let handle = std::thread::spawn(move || {
            let output = Command::new("cargo")
                .args(["run", "--release", "--example", "single_process_load_test"])
                .output()
                .expect("Failed to run load test");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Parse RPS from output
            for line in stdout.lines() {
                if line.contains("req/s") {
                    if let Some(rps_str) = line.split_whitespace().find(|s| s.parse::<f64>().is_ok()) {
                        if let Ok(rps) = rps_str.parse::<f64>() {
                            println!("  Process {}: {:.0} req/s", i, rps);
                            return rps;
                        }
                    }
                }
            }
            0.0
        });
        handles.push(handle);
    }

    // Collect results
    let mut total_rps = 0.0;
    for handle in handles {
        total_rps += handle.join().unwrap_or(0.0);
    }

    let elapsed = start.elapsed();

    println!();
    println!("📊 Aggregated Results");
    println!("  Processes: {}", num_processes);
    println!("  Total Duration: {:.2}s", elapsed.as_secs_f64());
    println!("  Aggregate Throughput: {:.0} req/s", total_rps);

    Ok(())
}
