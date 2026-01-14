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

## Testing with the provided example

The example client connects to the running server to demonstrate usage:

```bash
# Terminal 1: Start the server
cargo run

# Terminal 2: Run the example client
cargo run --example client
```
```

## Project Structure

- `proto/` - Protocol Buffer definitions
- `src/` - Rust source code
- `build.rs` - Build script to compile proto files
- `examples/` - Example client binaries (run via `cargo run --example client`)
