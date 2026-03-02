use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

const DEFAULT_MAX_REQUESTS: u32 = 100;
const DEFAULT_WINDOW_SECS: u64 = 60;

pub struct RateLimiter {
    requests: HashMap<IpAddr, (u32, Instant)>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            max_requests,
            window_secs,
        }
    }

    pub fn default() -> Self {
        RateLimiter::new(DEFAULT_MAX_REQUESTS, DEFAULT_WINDOW_SECS)
    }

    pub fn check(&mut self, addr: IpAddr) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let entry = self.requests.entry(addr).or_insert((0, now));

        if now.duration_since(entry.1) >= window {
            *entry = (1, now);
            true
        } else {
            entry.0 += 1;
            entry.0 <= self.max_requests
        }
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);
        self.requests.retain(|_, (_, ts)| now.duration_since(*ts) < window);
    }
}

pub struct CorsConfig {
    pub allowed_origins: String,
    pub allowed_methods: String,
    pub allowed_headers: String,
}

impl CorsConfig {
    pub fn permissive() -> Self {
        CorsConfig {
            allowed_origins: "*".to_string(),
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, PATCH".to_string(),
            allowed_headers: "Content-Type, Authorization, Accept".to_string(),
        }
    }

    pub fn restricted(origin: &str) -> Self {
        CorsConfig {
            allowed_origins: origin.to_string(),
            allowed_methods: "GET, POST, OPTIONS".to_string(),
            allowed_headers: "Content-Type, Authorization".to_string(),
        }
    }
}
