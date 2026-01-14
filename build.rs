fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .expect("failed to fetch vendored protoc");
    std::env::set_var("PROTOC", protoc);

    tonic_build::configure()
        .file_descriptor_set_path("proto/descriptor.bin")
        .compile(&["proto/rate_limiter.proto"], &["proto"])
        .expect("failed to compile protos");
}
