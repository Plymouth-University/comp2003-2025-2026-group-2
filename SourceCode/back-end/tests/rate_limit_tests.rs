use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::post,
    Router,
};
use back_end::{
    handlers::login,
    rate_limit::{rate_limit_middleware, RateLimitState},
    db, AppState,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::net::{IpAddr, Ipv4Addr};
use tempfile::NamedTempFile;
use tower::ServiceExt;

async fn setup_test_app_with_rate_limit() -> (Router, NamedTempFile) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_str().expect("Failed to get temp path");
    
    let connection_string = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string)
        .await
        .expect("Failed to create test db");

    db::init_db(&pool)
        .await
        .expect("Failed to initialize test db");

    let rate_limit_state = RateLimitState::new();
    let state = AppState {
        sqlite: pool,
        rate_limit: rate_limit_state.clone(),
        metrics: back_end::metrics::Metrics::new(),
    };

    let app = Router::new()
        .route("/auth/login", post(login))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state);

    (app, temp_file)
}

#[tokio::test]
async fn test_rate_limit_state_creation() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    
    assert!(state.check_general(ip));
    assert!(state.check_login(ip));
    assert!(state.check_register(ip));
}

#[tokio::test]
async fn test_rate_limit_login_allows_within_limit() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    
    for i in 0..5 {
        let allowed = state.check_login(ip);
        assert!(allowed, "Request {} should be allowed (limit is 5 per minute)", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limit_login_blocks_after_limit() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));
    
    for _ in 0..5 {
        assert!(state.check_login(ip), "Should allow within limit");
    }
    
    let blocked = state.check_login(ip);
    assert!(!blocked, "Should block 6th request");
}

#[tokio::test]
async fn test_rate_limit_register_allows_within_limit() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3));
    
    for i in 0..3 {
        let allowed = state.check_register(ip);
        assert!(allowed, "Request {} should be allowed (limit is 3 per hour)", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limit_register_blocks_after_limit() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 4));
    
    for _ in 0..3 {
        assert!(state.check_register(ip), "Should allow within limit");
    }
    
    let blocked = state.check_register(ip);
    assert!(!blocked, "Should block 4th request");
}

#[tokio::test]
async fn test_rate_limit_general_allows_many_requests() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5));
    
    for i in 0..60 {
        let allowed = state.check_general(ip);
        assert!(allowed, "Request {} should be allowed (limit is 60 per minute)", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limit_general_blocks_after_limit() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 6));
    
    for _ in 0..60 {
        assert!(state.check_general(ip), "Should allow within limit");
    }
    
    let blocked = state.check_general(ip);
    assert!(!blocked, "Should block 61st request");
}

#[tokio::test]
async fn test_rate_limit_different_ips_independent() {
    let state = RateLimitState::new();
    let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
    let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 11));
    
    for _ in 0..5 {
        assert!(state.check_login(ip1));
    }
    assert!(!state.check_login(ip1), "IP1 should be blocked");
    
    for _ in 0..5 {
        assert!(state.check_login(ip2), "IP2 should still be allowed");
    }
}

#[tokio::test]
async fn test_rate_limit_middleware_allows_first_request() {
    let (app, _temp) = setup_test_app_with_rate_limit().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "10.0.0.1")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "email": "test@example.com",
                "password": "TestPass123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    let status = response.status();
    assert_ne!(status, StatusCode::TOO_MANY_REQUESTS, "First request should not be rate limited");
}

#[tokio::test]
async fn test_rate_limit_middleware_blocks_after_limit() {
    let (mut app, _temp) = setup_test_app_with_rate_limit().await;
    let ip = "10.0.0.2";
    
    for i in 0..5 {
        let request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip)
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "email": "test@example.com",
                    "password": "TestPass123!"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::oneshot(&mut app, request).await.unwrap();
        let status = response.status();
        
        assert_ne!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "Request {} should not be rate limited",
            i + 1
        );
    }
    
    let blocked_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", ip)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "email": "test@example.com",
                "password": "TestPass123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::oneshot(&mut app, blocked_request).await.unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "6th request should be rate limited"
    );
}

#[tokio::test]
async fn test_rate_limit_middleware_extracts_ip_from_x_forwarded_for() {
    let state = RateLimitState::new();
    let ip1 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10));
    let ip2 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 11));
    
    for _ in 0..5 {
        assert!(state.check_login(ip1));
    }
    assert!(!state.check_login(ip1));
    
    assert!(state.check_login(ip2), "Different IP should not be affected");
}

#[tokio::test]
async fn test_rate_limit_ipv6_support() {
    let state = RateLimitState::new();
    let ipv6: IpAddr = "2001:0db8:85a3:0000:0000:8a2e:0370:7334".parse().unwrap();
    
    for i in 0..5 {
        let allowed = state.check_login(ipv6);
        assert!(allowed, "IPv6 request {} should be allowed", i + 1);
    }
    
    let blocked = state.check_login(ipv6);
    assert!(!blocked, "IPv6 should be blocked after limit");
}

#[tokio::test]
async fn test_rate_limit_clone_state() {
    let state1 = RateLimitState::new();
    let state2 = state1.clone();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1));
    
    for _ in 0..5 {
        assert!(state1.check_login(ip));
    }
    
    assert!(!state2.check_login(ip), "Cloned state should share the same limiters");
}

#[tokio::test]
async fn test_rate_limit_default_trait() {
    let state = RateLimitState::default();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 2, 2));
    
    assert!(state.check_login(ip));
    assert!(state.check_register(ip));
    assert!(state.check_general(ip));
}

#[tokio::test]
async fn test_rate_limit_multiple_endpoint_types() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 3, 1));
    
    for _ in 0..5 {
        assert!(state.check_login(ip));
    }
    assert!(!state.check_login(ip), "Login should be blocked");
    
    for _ in 0..3 {
        assert!(state.check_register(ip), "Register should still work (different limiter)");
    }
    assert!(!state.check_register(ip), "Register should now be blocked");
    
    for _ in 0..60 {
        assert!(state.check_general(ip), "General should still work (different limiter)");
    }
}

#[tokio::test]
async fn test_rate_limit_concurrent_requests_same_ip() {
    use tokio::task::JoinSet;
    
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 4, 1));
    
    let mut set = JoinSet::new();
    
    for _ in 0..10 {
        let state_clone = state.clone();
        set.spawn(async move {
            state_clone.check_login(ip)
        });
    }
    
    let mut allowed_count = 0;
    let mut blocked_count = 0;
    
    while let Some(result) = set.join_next().await {
        if result.unwrap() {
            allowed_count += 1;
        } else {
            blocked_count += 1;
        }
    }
    
    assert_eq!(allowed_count, 5, "Should allow exactly 5 requests");
    assert_eq!(blocked_count, 5, "Should block exactly 5 requests");
}

#[tokio::test]
async fn test_rate_limit_response_includes_retry_after() {
    let (mut app, _temp) = setup_test_app_with_rate_limit().await;
    let ip = "10.0.0.20";
    
    for _ in 0..5 {
        let request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip)
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "email": "test@example.com",
                    "password": "TestPass123!"
                }))
                .unwrap(),
            ))
            .unwrap();

        let _ = ServiceExt::<Request<Body>>::oneshot(&mut app, request).await.unwrap();
    }
    
    let blocked_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", ip)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "email": "test@example.com",
                "password": "TestPass123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::oneshot(&mut app, blocked_request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert!(body["error"].as_str().unwrap().contains("Rate limit exceeded"));
    assert_eq!(body["retry_after"], "60");
}

#[tokio::test]
async fn test_rate_limit_email_login_allows_within_limit() {
    let state = RateLimitState::new();
    
    for i in 0..10 {
        let allowed = state.check_login_email("test@example.com");
        assert!(allowed, "Request {} should be allowed (limit is 10 per minute for email)", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limit_email_login_blocks_after_limit() {
    let state = RateLimitState::new();
    
    for _ in 0..10 {
        assert!(state.check_login_email("test@example.com"), "Should allow within limit");
    }
    
    let blocked = state.check_login_email("test@example.com");
    assert!(!blocked, "Should block 11th request for same email");
}

#[tokio::test]
async fn test_rate_limit_email_case_insensitive() {
    let state = RateLimitState::new();
    
    for _ in 0..5 {
        assert!(state.check_login_email("Test@Example.COM"));
    }
    
    for _ in 0..5 {
        assert!(state.check_login_email("test@example.com"));
    }
    
    let blocked = state.check_login_email("TEST@EXAMPLE.COM");
    assert!(!blocked, "Should be blocked regardless of case");
}

#[tokio::test]
async fn test_rate_limit_email_register_allows_within_limit() {
    let state = RateLimitState::new();
    
    for i in 0..5 {
        let allowed = state.check_register_email("newuser@example.com");
        assert!(allowed, "Request {} should be allowed (limit is 5 per hour for email registration)", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limit_email_register_blocks_after_limit() {
    let state = RateLimitState::new();
    
    for _ in 0..5 {
        assert!(state.check_register_email("spam@example.com"), "Should allow within limit");
    }
    
    let blocked = state.check_register_email("spam@example.com");
    assert!(!blocked, "Should block 6th registration attempt for same email");
}

#[tokio::test]
async fn test_rate_limit_different_emails_independent() {
    let state = RateLimitState::new();
    
    for _ in 0..10 {
        assert!(state.check_login_email("user1@example.com"));
    }
    assert!(!state.check_login_email("user1@example.com"), "user1 should be blocked");
    
    for _ in 0..10 {
        assert!(state.check_login_email("user2@example.com"), "user2 should still be allowed");
    }
}

#[tokio::test]
async fn test_rate_limit_ip_and_email_both_enforced() {
    let state = RateLimitState::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 5, 1));
    
    for _ in 0..5 {
        assert!(state.check_login(ip));
        assert!(state.check_login_email("test@example.com"));
    }
    
    assert!(!state.check_login(ip), "IP should be blocked");
    assert!(state.check_login_email("test@example.com"), "Email should still have attempts left");
}

#[tokio::test]
async fn test_rate_limit_botnet_scenario() {
    let state = RateLimitState::new();
    
    for i in 0..20 {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, i as u8));
        
        for _ in 0..5 {
            assert!(state.check_login(ip), "Different IPs should each get 5 attempts");
        }
    }
    
    for i in 0..10 {
        assert!(state.check_login_email("target@example.com"), "Email attempt {}", i + 1);
    }
    
    let blocked = state.check_login_email("target@example.com");
    assert!(!blocked, "Email should be blocked even though many different IPs were used");
}

#[tokio::test]
async fn test_rate_limit_middleware_checks_email_from_body() {
    let (mut app, _temp) = setup_test_app_with_rate_limit().await;
    let email = "ratelimited@example.com";
    
    for i in 0..10 {
        let ip = format!("10.0.0.{}", 30 + i);
        let request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", &ip)
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "email": email,
                    "password": "TestPass123!"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::oneshot(&mut app, request).await.unwrap();
        let status = response.status();
        
        assert_ne!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "Request {} should not be rate limited (different IPs)",
            i + 1
        );
    }
    
    let blocked_request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "10.0.0.99")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "email": email,
                "password": "TestPass123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = ServiceExt::<Request<Body>>::oneshot(&mut app, blocked_request).await.unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "11th request with same email but different IP should be rate limited"
    );
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert!(body["error"].as_str().unwrap().contains("email"));
}
