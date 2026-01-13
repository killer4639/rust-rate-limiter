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

The server will start listening on `[::1]:50051` (IPv6 localhost on port 50051).

## Testing with grpcurl

```bash
# Install grpcurl if not already installed
# https://github.com/fullstorydev/grpcurl/releases

# Send a ping request
grpcurl -plaintext -d '{"message":"hello"}' [::1]:50051 rate_limiter.RateLimiter/Ping

## Testing with the provided integration test

The integration test connects to the running server and verifies the `Ping` method works:

```bash
# Terminal 1: Start the server
cargo run

# Terminal 2: Run the integration test
cargo test --test client_integration -- --nocapture
```
```

## Project Structure

- `proto/` - Protocol Buffer definitions
- `src/` - Rust source code
- `build.rs` - Build script to compile proto files
- `tests/` - Integration tests (run via `cargo test`)
