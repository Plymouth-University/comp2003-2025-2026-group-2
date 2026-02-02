use axum::{
    Json,
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, direct::NotKeyed},
};
use serde_json::json;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

type IpLimiter = DashMap<
    IpAddr,
    (
        Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
        Instant,
    ),
>;
type StringLimiter = DashMap<
    String,
    (
        Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
        Instant,
    ),
>;

#[derive(Clone)]
pub struct RateLimitState {
    pub ip_login_limiter: Arc<IpLimiter>,
    pub ip_register_limiter: Arc<IpLimiter>,
    pub ip_general_limiter: Arc<IpLimiter>,
    pub email_login_limiter: Arc<StringLimiter>,
    pub email_register_limiter: Arc<StringLimiter>,
    pub disabled: bool,
}

impl RateLimitState {
    pub fn new() -> Self {
        let disabled = std::env::var("DISABLE_RATE_LIMIT")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        if disabled {
            tracing::warn!("Rate limiting is DISABLED via DISABLE_RATE_LIMIT environment variable");
        }

        Self {
            ip_login_limiter: Arc::new(DashMap::new()),
            ip_register_limiter: Arc::new(DashMap::new()),
            ip_general_limiter: Arc::new(DashMap::new()),
            email_login_limiter: Arc::new(DashMap::new()),
            email_register_limiter: Arc::new(DashMap::new()),
            disabled,
        }
    }

    #[must_use]
    pub fn disabled() -> Self {
        Self {
            ip_login_limiter: Arc::new(DashMap::new()),
            ip_register_limiter: Arc::new(DashMap::new()),
            ip_general_limiter: Arc::new(DashMap::new()),
            email_login_limiter: Arc::new(DashMap::new()),
            email_register_limiter: Arc::new(DashMap::new()),
            disabled: true,
        }
    }

    fn get_or_create_ip_limiter(
        map: &IpLimiter,
        ip: IpAddr,
        quota: Quota,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let now = Instant::now();
        map.entry(ip)
            .or_insert_with(|| (Arc::new(RateLimiter::direct(quota)), now))
            .0
            .clone()
    }

    fn get_or_create_string_limiter(
        map: &StringLimiter,
        key: String,
        quota: Quota,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let now = Instant::now();
        map.entry(key)
            .or_insert_with(|| (Arc::new(RateLimiter::direct(quota)), now))
            .0
            .clone()
    }

    pub fn cleanup_expired_entries(&self, max_age: std::time::Duration) {
        let now = Instant::now();

        self.ip_login_limiter
            .retain(|_, (_, last_access)| now.duration_since(*last_access) < max_age);

        self.ip_register_limiter
            .retain(|_, (_, last_access)| now.duration_since(*last_access) < max_age);

        self.ip_general_limiter
            .retain(|_, (_, last_access)| now.duration_since(*last_access) < max_age);

        self.email_login_limiter
            .retain(|_, (_, last_access)| now.duration_since(*last_access) < max_age);

        self.email_register_limiter
            .retain(|_, (_, last_access)| now.duration_since(*last_access) < max_age);
    }

    pub fn spawn_cleanup_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                self.cleanup_expired_entries(std::time::Duration::from_secs(3600));
                tracing::debug!("Rate limiter cleanup completed");
            }
        });
    }

    /// Checks if a login attempt from a given IP is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_login(&self, ip: IpAddr) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_login_limiter, ip, quota);
        limiter.check().is_ok()
    }

    /// Checks if a login attempt for a given email is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_login_email(&self, email: &str) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        let limiter = Self::get_or_create_string_limiter(
            &self.email_login_limiter,
            email.to_lowercase(),
            quota,
        );
        limiter.check().is_ok()
    }

    /// Checks if a registration attempt from a given IP is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_register(&self, ip: IpAddr) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_register_limiter, ip, quota);
        limiter.check().is_ok()
    }

    /// Checks if a registration attempt for a given email is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_register_email(&self, email: &str) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(20).unwrap());
        let limiter = Self::get_or_create_string_limiter(
            &self.email_register_limiter,
            email.to_lowercase(),
            quota,
        );
        limiter.check().is_ok()
    }

    /// Checks if a general request from a given IP is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_general(&self, ip: IpAddr) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(60).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_general_limiter, ip, quota);
        limiter.check().is_ok()
    }

    /// Checks if an OAuth request from a given IP is allowed by rate limits.
    ///
    /// # Panics
    /// Panics if the hardcoded quota is invalid.
    #[must_use]
    pub fn check_oauth(&self, ip: IpAddr) -> bool {
        if self.disabled {
            return true;
        }
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_general_limiter, ip, quota);
        limiter.check().is_ok()
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_email_from_body(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("email")
                .and_then(|e| e.as_str().map(std::string::ToString::to_string))
        })
}

pub async fn rate_limit_middleware(
    State(app_state): State<crate::AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| addr.ip());

    if app_state.rate_limit.disabled {
        tracing::debug!("Rate limiting disabled, allowing request from {}", ip);
        return next.run(req).await;
    }

    let path = req.uri().path().to_string();

    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();

    let email = extract_email_from_body(&body_bytes);

    let ip_allowed = if path.contains("/auth/login") {
        app_state.rate_limit.check_login(ip)
    } else if path.contains("/auth/register") {
        app_state.rate_limit.check_register(ip)
    } else if path.contains("/auth/google/") || path.contains("/auth/oauth/") {
        app_state.rate_limit.check_oauth(ip)
    } else {
        app_state.rate_limit.check_general(ip)
    };

    if !ip_allowed {
        app_state.metrics.increment_rate_limit_hits();
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Rate limit exceeded for your IP address. Please try again later.",
                "retry_after": "60"
            })),
        )
            .into_response();
    }

    if let Some(email_str) = email.as_ref() {
        let email_allowed = if path.contains("/auth/login") {
            app_state.rate_limit.check_login_email(email_str)
        } else if path.contains("/auth/register") {
            app_state.rate_limit.check_register_email(email_str)
        } else {
            true
        };

        if !email_allowed {
            app_state.metrics.increment_rate_limit_hits();
            tracing::warn!("Rate limit exceeded for email: {}", email_str);
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Rate limit exceeded for this email address. Please try again later.",
                    "retry_after": if path.contains("/auth/login") { "60" } else { "3600" }
                })),
            )
                .into_response();
        }
    }

    let new_body = axum::body::Body::from(body_bytes);
    let req = Request::from_parts(parts, new_body);

    next.run(req).await
}
