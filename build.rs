fn main() {
    tonic_build::compile_protos("proto/rate_limiter.proto")
        .expect("failed to compile protos");
}
