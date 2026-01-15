use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::{Request, Response, Status};

use crate::rate_limiter::rate_limiter_server::RateLimiter;
use crate::rate_limiter::{RateLimitRequest, RateLimitResponse, HeartBeatRequest, HeartBeatResponse};

struct TokenBucket {
    tokens: i32,
    last_refill: Instant,
}

pub struct RateLimiterService {
    // Shared state across all requests
    state: Arc<DashMap<String, TokenBucket>>,
    // Configuration
    tokens_per_window: i32,
    window_duration: Duration,
}

impl Default for RateLimiterService {
    fn default() -> Self {
        Self {
            state: Arc::new(DashMap::new()),
            tokens_per_window: 10,                    // 10 tokens per window
            window_duration: Duration::from_secs(60), // 1 minute window
        }
    }
}

impl RateLimiterService {
    fn validate_and_normalize_request(&self, req: &RateLimitRequest) -> Result<i32, Status> {
        // Validate that id is present
        if req.id.is_empty() {
            return Err(Status::invalid_argument("id is required"));
        }

        // Default tokens_requested to 1 if not provided or invalid
        let tokens = if req.tokens_requested <= 0 {
            1
        } else {
            req.tokens_requested
        };

        Ok(tokens)
    }

    fn check_rate_limit(&self, id: &str, tokens_requested: i32) -> Result<bool, Status> {
        let now = Instant::now();

        // Get or insert bucket for this ID
        let mut bucket = self
            .state
            .entry(id.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.tokens_per_window,
                last_refill: now,
            });

        // Refill tokens if window has elapsed
        let elapsed = now.duration_since(bucket.last_refill);
        if elapsed >= self.window_duration {
            bucket.tokens = self.tokens_per_window;
            bucket.last_refill = now;
        }

        // Check if enough tokens available
        if bucket.tokens >= tokens_requested {
            bucket.tokens -= tokens_requested;
            Ok(true) // Allowed
        } else {
            Ok(false) // Rate limited
        }
    }
}

#[tonic::async_trait]
impl RateLimiter for RateLimiterService {
    async fn check_rate_limit(
        &self,
        request: Request<RateLimitRequest>,
    ) -> Result<Response<RateLimitResponse>, Status> {
        let req = request.into_inner();

        let tokens = self.validate_and_normalize_request(&req)?;

        // Check rate limit
        let allowed = self.check_rate_limit(&req.id, tokens)?;

        if allowed {
            tracing::info!("Rate limit ALLOWED - id: {}, tokens: {}", req.id, tokens);

            let reply = RateLimitResponse {
                status: "success".to_string(),
            };

            Ok(Response::new(reply))
        } else {
            tracing::warn!("Rate limit EXCEEDED - id: {}, tokens: {}", req.id, tokens);

            Err(Status::resource_exhausted(format!(
                "Rate limit exceeded for id: {}",
                req.id
            )))
        }
    }

    async fn heart_beat(
        &self,
        request: Request<HeartBeatRequest>,
    ) -> Result<Response<HeartBeatResponse>, Status> {
        Ok(Response::new(HeartBeatResponse {}))
    }
}
