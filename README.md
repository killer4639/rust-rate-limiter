# Rust Rate Limiter

A gRPC-based rate limiter service built with Rust, Tonic, and Tokio.

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

The server will start listening on `127.0.0.1:50051` (IPv4 localhost on port 50051).

## Testing with grpcurl

```bash
# Install grpcurl if not already installed
# https://github.com/fullstorydev/grpcurl/releases

# Send a ping request (reflection enabled, no .proto needed)
grpcurl -plaintext -d '{\"message\":\"hello\"}' 127.0.0.1:50051 rate_limiter.RateLimiter/Ping


## Example Clients and Load Tests

The `examples/` directory contains several binaries for testing and benchmarking:

- **client.rs**: Simple gRPC client for manual requests.
- **load_test.rs**: Multi-threaded, configurable load test for rate limiter and heartbeat endpoints.
- **loop_load_test.rs**: Repeated requests in a loop (single-threaded).
- **single_threaded_load_test.rs**: Basic single-threaded load test for baseline throughput.

### Running the Server
```bash
cargo run
```

### Running the Example Client
```bash
cargo run --example client
```

### Running the Main Load Test
```bash
# Run rate limiter endpoint load test
cargo run --example load_test -- rate

# Run heartbeat endpoint load test
cargo run --example load_test -- heartbeat

# Run both endpoints
cargo run --example load_test -- both
```

### Running Other Load Test Variants
```bash
cargo run --example loop_load_test
cargo run --example single_threaded_load_test
```

## Project Structure

- `proto/` - Protocol Buffer definitions
- `src/` - Rust source code
- `build.rs` - Build script to compile proto files
- `examples/` - Example client binaries (run via `cargo run --example client`)
