use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter entry for tracking requests per IP
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Simple in-memory rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Map of IP address to rate limit entry
    entries: Arc<Mutex<HashMap<IpAddr, RateLimitEntry>>>,
    /// Maximum requests per window
    max_requests: u32,
    /// Time window duration
    window_duration: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
        }
    }

    /// Check if IP is allowed to make a request
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), String> {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();

        let entry = entries.entry(ip).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Check if window has expired
        if now.duration_since(entry.window_start) > self.window_duration {
            // Reset window
            entry.count = 0;
            entry.window_start = now;
        }

        // Check if limit exceeded
        if entry.count >= self.max_requests {
            let retry_after = self.window_duration
                .checked_sub(now.duration_since(entry.window_start))
                .unwrap_or(Duration::from_secs(0));

            return Err(format!(
                "Rate limit exceeded. Try again in {} seconds",
                retry_after.as_secs()
            ));
        }

        // Increment counter
        entry.count += 1;
        Ok(())
    }

    /// Clean up old entries (called periodically by background task)
    pub async fn cleanup(&self) {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();

        entries.retain(|_, entry| {
            now.duration_since(entry.window_start) <= self.window_duration
        });
    }
}

/// Extract IP address from request
fn extract_ip(request: &Request) -> IpAddr {
    // Try to get real IP from X-Forwarded-For header (for proxies)
    if let Some(forwarded_for) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(ip_str) = forwarded_for.split(',').next() {
            if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Try to get from X-Real-IP header
    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        if let Ok(ip) = real_ip.parse::<IpAddr>() {
            return ip;
        }
    }

    // Fallback to localhost (when running locally or can't determine IP)
    "127.0.0.1".parse().unwrap()
}

/// Extension wrapper for RateLimiter
#[derive(Clone)]
pub struct RateLimiterExtension(pub Arc<RateLimiter>);

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    axum::Extension(rate_limiter): axum::Extension<RateLimiterExtension>,
    request: Request,
    next: Next,
) -> Response {
    let ip = extract_ip(&request);

    match rate_limiter.0.check_rate_limit(ip).await {
        Ok(_) => next.run(request).await,
        Err(msg) => (
            StatusCode::TOO_MANY_REQUESTS,
            [(
                axum::http::header::RETRY_AFTER,
                "60", // Suggest retry after 60 seconds
            )],
            msg,
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(3, 60);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit(ip).await.is_ok());
        assert!(limiter.check_rate_limit(ip).await.is_ok());
        assert!(limiter.check_rate_limit(ip).await.is_ok());

        // 4th request should fail
        assert!(limiter.check_rate_limit(ip).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, 60);
        let ip1: IpAddr = "127.0.0.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.1".parse().unwrap();

        // Both IPs should have separate limits
        assert!(limiter.check_rate_limit(ip1).await.is_ok());
        assert!(limiter.check_rate_limit(ip1).await.is_ok());
        assert!(limiter.check_rate_limit(ip2).await.is_ok());
        assert!(limiter.check_rate_limit(ip2).await.is_ok());

        // Both should now be at limit
        assert!(limiter.check_rate_limit(ip1).await.is_err());
        assert!(limiter.check_rate_limit(ip2).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let limiter = RateLimiter::new(5, 1); // 1 second window
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Make some requests
        limiter.check_rate_limit(ip).await.ok();
        limiter.check_rate_limit(ip).await.ok();

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup should remove old entries
        limiter.cleanup().await;

        // Should be able to make requests again
        assert!(limiter.check_rate_limit(ip).await.is_ok());
    }
}
