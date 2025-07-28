use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use axum::{
    middleware::Next,
    response::Response,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), StatusCode> {
        let now = Instant::now();
        let mut requests = self.requests.write().await;
        
        let client_requests = requests.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        client_requests.retain(|&req_time| now.duration_since(req_time) < self.window);
        
        if client_requests.len() >= self.max_requests {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        
        client_requests.push(now);
        Ok(())
    }

    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut requests = self.requests.write().await;
        
        requests.retain(|_, times| {
            times.retain(|&req_time| now.duration_since(req_time) < self.window);
            !times.is_empty()
        });
    }
}

pub async fn rate_limit_middleware<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<B>,
    next: Next<B>,
    rate_limiter: Arc<RateLimiter>,
) -> Result<Response, StatusCode> {
    let client_id = addr.ip().to_string();
    
    rate_limiter.check_rate_limit(&client_id).await?;
    
    Ok(next.run(req).await)
}

// DDoS protection with circuit breaker
pub struct CircuitBreaker {
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
    failures: Arc<RwLock<HashMap<String, CircuitState>>>,
}

#[derive(Clone)]
struct CircuitState {
    failures: usize,
    last_failure: Option<Instant>,
    state: BreakerState,
}

#[derive(Clone, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, success_threshold: usize, timeout_seconds: u64) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout: Duration::from_secs(timeout_seconds),
            failures: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn call<F, T, E>(&self, key: &str, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<&'static str>,
    {
        let mut states = self.failures.write().await;
        let state = states.entry(key.to_string()).or_insert(CircuitState {
            failures: 0,
            last_failure: None,
            state: BreakerState::Closed,
        });

        match state.state {
            BreakerState::Open => {
                if let Some(last_failure) = state.last_failure {
                    if Instant::now().duration_since(last_failure) > self.timeout {
                        state.state = BreakerState::HalfOpen;
                    } else {
                        return Err(E::from("Circuit breaker is open"));
                    }
                }
            }
            _ => {}
        }

        match f() {
            Ok(result) => {
                if state.state == BreakerState::HalfOpen {
                    state.failures = 0;
                    state.state = BreakerState::Closed;
                }
                Ok(result)
            }
            Err(e) => {
                state.failures += 1;
                state.last_failure = Some(Instant::now());
                
                if state.failures >= self.failure_threshold {
                    state.state = BreakerState::Open;
                }
                
                Err(e)
            }
        }
    }
}