/// Generated from proto/rate_limiter.proto
/// Manual proto definitions to avoid needing protoc installed

use prost::Message;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RateLimitRequest {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RateLimitResponse {
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
