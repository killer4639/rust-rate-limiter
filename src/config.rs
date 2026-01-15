/// Configuration module for load testing and server settings
use std::env;

#[derive(Clone, Debug)]
pub struct LoadTestConfig {
    pub num_threads: usize,
    pub requests_per_thread: usize,
    pub server_url: String,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            num_threads: 50,
            requests_per_thread: 100,
            server_url: "http://127.0.0.1:50051".to_string(),
        }
    }
}

impl LoadTestConfig {
    /// Load configuration from environment variables or use defaults
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let num_threads = env::var("THREADS")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<usize>()?;
        
        let requests_per_thread = env::var("REQUESTS_PER_THREAD")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()?;
        
        let server_url = env::var("SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
        
        Ok(Self {
            num_threads,
            requests_per_thread,
            server_url,
        })
    }

    pub fn total_requests(&self) -> usize {
        self.num_threads * self.requests_per_thread
    }

    pub fn print_summary(&self) {
        println!("🔥 Load Test Configuration");
        println!("  Threads: {}", self.num_threads);
        println!("  Requests per thread: {}", self.requests_per_thread);
        println!("  Total requests: {}", self.total_requests());
        println!("  Server: {}", self.server_url);
        println!();
    }
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let bind_address = env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse::<u16>()
            .unwrap_or(50051);
        
        Self {
            bind_address,
            port,
        }
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
}
