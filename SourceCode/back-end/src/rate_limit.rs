use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter,
};
use serde_json::json;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone)]
pub struct RateLimitState {
    pub ip_login_limiter: Arc<DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    pub ip_register_limiter: Arc<DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    pub ip_general_limiter: Arc<DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    pub email_login_limiter: Arc<DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    pub email_register_limiter: Arc<DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self {
            ip_login_limiter: Arc::new(DashMap::new()),
            ip_register_limiter: Arc::new(DashMap::new()),
            ip_general_limiter: Arc::new(DashMap::new()),
            email_login_limiter: Arc::new(DashMap::new()),
            email_register_limiter: Arc::new(DashMap::new()),
        }
    }

    fn get_or_create_ip_limiter(
        map: &DashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
        ip: IpAddr,
        quota: Quota,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        map.entry(ip)
            .or_insert_with(|| Arc::new(RateLimiter::direct(quota)))
            .clone()
    }

    fn get_or_create_string_limiter(
        map: &DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
        key: String,
        quota: Quota,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        map.entry(key)
            .or_insert_with(|| Arc::new(RateLimiter::direct(quota)))
            .clone()
    }

    pub fn check_login(&self, ip: IpAddr) -> bool {
        let quota = Quota::per_minute(NonZeroU32::new(5).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_login_limiter, ip, quota);
        limiter.check().is_ok()
    }

    pub fn check_login_email(&self, email: &str) -> bool {
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        let limiter = Self::get_or_create_string_limiter(&self.email_login_limiter, email.to_lowercase(), quota);
        limiter.check().is_ok()
    }

    pub fn check_register(&self, ip: IpAddr) -> bool {
        let quota = Quota::per_hour(NonZeroU32::new(3).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_register_limiter, ip, quota);
        limiter.check().is_ok()
    }

    pub fn check_register_email(&self, email: &str) -> bool {
        let quota = Quota::per_hour(NonZeroU32::new(5).unwrap());
        let limiter = Self::get_or_create_string_limiter(&self.email_register_limiter, email.to_lowercase(), quota);
        limiter.check().is_ok()
    }

    pub fn check_general(&self, ip: IpAddr) -> bool {
        let quota = Quota::per_minute(NonZeroU32::new(60).unwrap());
        let limiter = Self::get_or_create_ip_limiter(&self.ip_general_limiter, ip, quota);
        limiter.check().is_ok()
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_ip_from_request(req: &Request) -> Option<IpAddr> {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            req.extensions()
                .get::<std::net::SocketAddr>()
                .map(|addr| addr.ip())
        })
}

fn extract_email_from_body(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|json| json.get("email").and_then(|e| e.as_str().map(|s| s.to_string())))
}

pub async fn rate_limit_middleware(
    State(rate_limit_state): State<RateLimitState>,
    req: Request,
    next: Next,
) -> Response {
    let ip = match extract_ip_from_request(&req) {
        Some(ip) => ip,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Cannot determine client IP address" })),
            )
                .into_response();
        }
    };

    let path = req.uri().path().to_string();
    
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();
    
    let email = extract_email_from_body(&body_bytes);
    
    let ip_allowed = if path.contains("/auth/login") {
        rate_limit_state.check_login(ip)
    } else if path.contains("/auth/register") {
        rate_limit_state.check_register(ip)
    } else {
        rate_limit_state.check_general(ip)
    };

    if !ip_allowed {
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
            rate_limit_state.check_login_email(email_str)
        } else if path.contains("/auth/register") {
            rate_limit_state.check_register_email(email_str)
        } else {
            true
        };

        if !email_allowed {
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
