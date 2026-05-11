# LogSmart Backend - Test Coverage Implementation Plan

> **For agentic workers:** Use subagent-driven-development to implement test groups in parallel. Each group is independent and can be tested in isolation.

**Goal:** Create 84 new tests covering 30 critical untested code paths and close security/data integrity gaps.

**Architecture:** Test groups organized by risk tier (Security → Business Logic → Data Consistency → Operations). Each tier can be implemented independently. Tests use existing test infrastructure (common::setup_test_db, factories, mocks).

**Tech Stack:** Rust, Tokio for async, SQLx for database testing, Mockito for mocking, Proptest for property-based testing.

**Estimated Effort:** 150-200 hours over 4-5 weeks

---

## FILE STRUCTURE

### New Test Files to Create
- `tests/security_critical_tests.rs` - OAuth, Passkey, JWT, Rate Limiting tests
- `tests/business_logic_tests.rs` - Clock in/out, Templates, Invitations, Periods
- `tests/data_consistency_tests.rs` - Race conditions, Transactions, Cache invalidation
- `tests/authorization_tests.rs` - Cross-company access, Role overrides, Deleted user tokens

### Files Modified
- `back-end/src/services/oauth_service.rs` - Add JWKS cache time tracking
- `back-end/src/rate_limit.rs` - Fix X-Forwarded-For and email case sensitivity
- `back-end/src/middleware.rs` - Fix deleted user token validation
- `back-end/src/jwt_manager.rs` - Add `iat` future time validation

---

## TASK BREAKDOWN

## TIER 1: CRITICAL SECURITY TESTS (40 tests)

### Task 1: OAuth JWKS Cache Staleness Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/services/oauth_service.rs` (lines 114-148)
- Dependencies: Mock time, mock HTTP client

**Description:** Test that JWKS cache respects TTL expiration and refreshes keys.

- [ ] **Step 1: Create test file with OAuth mock setup**

```rust
#[tokio::test]
async fn test_oauth_jwks_cache_staleness() {
    // Test that JWKS cache expires after TTL
    // When cache TTL exceeded, new fetch occurs
    // Expired signing keys are NOT used for validation
}

#[tokio::test]
async fn test_oauth_jwks_concurrent_cache_refresh() {
    // Two concurrent requests for JWKS during cache TTL boundary
    // Should not cause duplicate fetches
    // Both should get fresh keys
}

#[tokio::test]
async fn test_oauth_jwks_refresh_on_invalid_key() {
    // Token signed with expired key
    // Cache misses key
    // Cache should refresh automatically
    // Old key should NOT validate new token
}
```

- [ ] **Step 2: Implement cache TTL tracking in oauth_service.rs**

Modify `get_jwks()` to track last fetch time and compare against TTL.

- [ ] **Step 3: Run tests and verify they pass**

`cargo test --test security_critical_tests test_oauth_jwks`

- [ ] **Step 4: Commit**

`git commit -m "test: add OAuth JWKS cache staleness tests"`

---

### Task 2: OAuth Duplicate User Creation (Race Condition) Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/services/oauth_service.rs` (lines 230-280)

**Description:** Test concurrent OAuth callbacks don't create duplicate users.

- [ ] **Step 1: Add race condition test**

```rust
#[tokio::test]
async fn test_oauth_concurrent_callbacks_same_email() {
    let pool = common::setup_test_db().await;
    
    // Simulate two concurrent OAuth callbacks for same email
    let oauth_provider = "google";
    let oauth_subject = "123456";
    let email = "concurrent@example.com";
    
    let callback1 = services::oauth_service::get_or_create_user(
        &pool, oauth_provider, oauth_subject, email, None
    );
    let callback2 = services::oauth_service::get_or_create_user(
        &pool, oauth_provider, oauth_subject, email, None
    );
    
    let (user1, user2) = tokio::join!(callback1, callback2);
    
    // Both should succeed
    assert!(user1.is_ok());
    assert!(user2.is_ok());
    
    // Both should have same user ID (no duplicates)
    assert_eq!(user1.unwrap().id, user2.unwrap().id);
    
    // Database should have exactly 1 user with this email
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    ).bind(email).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_oauth_account_linking_duplicate_prevention() {
    // User manually creates account, then OAuth attempts to link
    // Should detect conflict and handle gracefully
}

#[tokio::test]
async fn test_oauth_linking_existing_oauth_account() {
    // User already has OAuth account, tries to link same OAuth account
    // Should return existing user, not create duplicate
}
```

- [ ] **Step 2: Add database constraint or application-level duplicate prevention**

Ensure unique constraint on (oauth_provider, oauth_subject) or use SELECT FOR UPDATE.

- [ ] **Step 3: Run test to verify it passes**

`cargo test --test security_critical_tests test_oauth_concurrent_callbacks_same_email`

- [ ] **Step 4: Commit**

`git commit -m "test: add OAuth duplicate user creation race condition tests"`

---

### Task 3: Passkey Attestation Bypass Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/handlers/passkey_handlers.rs` (lines 176-313)

**Description:** Verify invalid passkey attestations are rejected.

- [ ] **Step 1: Create invalid attestation test cases**

```rust
#[tokio::test]
async fn test_passkey_attestation_invalid_format() {
    // Send malformed attestation object (not CBOR)
    // Should reject with clear error
    // User should NOT be created
}

#[tokio::test]
async fn test_passkey_attestation_missing_signature() {
    // Attestation object without signature
    // Should reject
}

#[tokio::test]
async fn test_passkey_attestation_signature_mismatch() {
    // Attestation signature doesn't match certificate
    // Should reject
}

#[tokio::test]
async fn test_passkey_credential_id_collision() {
    // Register same credential ID twice (same physical key)
    // Should detect and reject second registration
}

#[tokio::test]
async fn test_passkey_untrusted_attester() {
    // Attestation from untrusted attestation provider
    // Should reject (or handle as self-attestation)
}
```

- [ ] **Step 2: Verify attestation verification logic in passkey_handlers.rs**

Check that `verify_attestation()` properly validates all fields.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_passkey_attestation`

- [ ] **Step 4: Commit**

`git commit -m "test: add passkey attestation bypass prevention tests"`

---

### Task 4: Passkey Discoverable Authentication Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/handlers/passkey_handlers.rs` (lines 721-933)

**Description:** Test discoverable passkey login doesn't auth wrong user.

- [ ] **Step 1: Create credential collision tests**

```rust
#[tokio::test]
async fn test_passkey_discoverable_wrong_user_lookup() {
    let pool = common::setup_test_db().await;
    
    let user1 = common::UserFactory::create_basic();
    let user2 = common::UserFactory::create_basic();
    
    // Register discoverable passkey for user1
    let credential_id = "test-cred-123";
    let passkey1 = Passkey {
        credential_id: credential_id.to_string(),
        user_id: user1.id.clone(),
        ..create_test_passkey()
    };
    
    // Try to login with user2's ID but user1's credential
    // Should return user1, NOT user2
    let authenticated = handlers::passkey_handlers::finish_discoverable_passkey_login(
        &pool,
        credential_id,
        user2.id.clone(), // Wrong user
        &assertion_response,
    ).await;
    
    // Should either fail or return user1
    match authenticated {
        Ok(user) => assert_eq!(user.id, user1.id),
        Err(_) => {} // Also acceptable
    }
}

#[tokio::test]
async fn test_passkey_discoverable_user_id_mismatch() {
    // Assertion verification returns user1
    // But credential lookup returns user2
    // Should reject login
}
```

- [ ] **Step 2: Verify user ID validation in discoverable login**

Check that credential lookup matches request user ID.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_passkey_discoverable`

- [ ] **Step 4: Commit**

`git commit -m "test: add passkey discoverable auth validation tests"`

---

### Task 5: Deleted User Token Validity Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/middleware.rs` (lines 27-54)

**Description:** Verify deleted users with valid tokens are rejected.

- [ ] **Step 1: Create deleted user token tests**

```rust
#[tokio::test]
async fn test_deleted_user_token_rejected() {
    let pool = common::setup_test_db().await;
    
    let user = common::UserFactory::create_basic();
    let config = auth::JwtConfig::new("test_secret".to_string());
    let token = config.generate_token(&user.id, 24).unwrap();
    
    // Delete the user
    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(&user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Try to authenticate with old token
    let result = middleware::get_authenticated_user(&pool, &token, &config).await;
    
    // Should fail - user is deleted
    assert!(result.is_err());
}

#[tokio::test]
async fn test_deleted_user_cache_invalidation() {
    // User deleted
    // Cache should invalidate for that user
    // Subsequent requests should not return deleted user
}
```

- [ ] **Step 2: Modify middleware to check deleted_at during token auth**

Add explicit `deleted_at` check in `get_authenticated_user()`.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_deleted_user_token`

- [ ] **Step 4: Commit**

`git commit -m "test: add deleted user token validation tests"`

---

### Task 6: Rate Limit X-Forwarded-For Bypass Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/rate_limit.rs` (lines 280-359)

**Description:** Test X-Forwarded-For header manipulation doesn't bypass rate limits.

- [ ] **Step 1: Create header injection tests**

```rust
#[tokio::test]
async fn test_rate_limit_xforwardedfor_multiple_ips() {
    // Header: X-Forwarded-For: 127.0.0.1, 127.0.0.1, <attacker>
    // Should use first IP, not last (attacker)
}

#[tokio::test]
async fn test_rate_limit_xforwardedfor_ipv6_bypass() {
    // Header: X-Forwarded-For: ::ffff:127.0.0.1
    // Should correctly parse IPv6-mapped IPv4
}

#[tokio::test]
async fn test_rate_limit_xforwardedfor_empty_value() {
    // Header: X-Forwarded-For: (empty)
    // Should fall back to connection IP, not bypass
}

#[tokio::test]
async fn test_rate_limit_xforwardedfor_trusted_proxy() {
    // Different X-Forwarded-For with known proxy
    // Should still rate limit per IP correctly
}
```

- [ ] **Step 2: Fix X-Forwarded-For parsing in rate_limit.rs**

Use first IP only, validate format, handle IPv6 correctly.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_rate_limit_xforwardedfor`

- [ ] **Step 4: Commit**

`git commit -m "test: add X-Forwarded-For bypass prevention tests"`

---

### Task 7: Email Rate Limiting Case Sensitivity Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/rate_limit.rs` (lines 175-186)

**Description:** Verify email rate limiting is case-insensitive.

- [ ] **Step 1: Create case sensitivity tests**

```rust
#[tokio::test]
async fn test_email_rate_limit_case_insensitive() {
    let limiter = RateLimiter::new();
    
    // First attempt: test@example.com
    limiter.check_login_email("test@example.com").await.ok();
    
    // Second attempt: Test@example.com (uppercase T)
    // Should count as same email
    let result = limiter.check_login_email("Test@example.com").await;
    
    // Should be rate limited (same email)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_email_rate_limit_all_case_variants() {
    // test@example.com, TEST@EXAMPLE.COM, TeSt@ExAmPlE.cOm
    // All should hit same rate limit bucket
}
```

- [ ] **Step 2: Fix email normalization in rate_limit.rs**

Ensure all email comparisons use `.to_lowercase()`.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_email_rate_limit_case`

- [ ] **Step 4: Commit**

`git commit -m "test: add email rate limit case sensitivity tests"`

---

### Task 8: Deleted Company Token Validity Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/middleware.rs` (lines 32-54)

**Description:** Verify users of deleted companies are rejected.

- [ ] **Step 1: Create deleted company tests**

```rust
#[tokio::test]
async fn test_deleted_company_token_rejected() {
    let pool = common::setup_test_db().await;
    
    let company = common::CompanyFactory::create_basic();
    let user = UserRecord {
        company_id: Some(company.id.clone()),
        ..common::UserFactory::create_basic()
    };
    
    let config = auth::JwtConfig::new("test_secret".to_string());
    let token = config.generate_token(&user.id, 24).unwrap();
    
    // Soft-delete company
    sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Try to authenticate
    let result = middleware::get_authenticated_user(&pool, &token, &config).await;
    
    // Should fail
    assert!(result.is_err());
}
```

- [ ] **Step 2: Verify company_deleted_at check in middleware**

Ensure check happens even if user.deleted_at is None.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_deleted_company_token`

- [ ] **Step 4: Commit**

`git commit -m "test: add deleted company user rejection tests"`

---

### Task 9: JWT Future Issue Time Validation Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/jwt_manager.rs`

**Description:** Verify JWT tokens with future `iat` claim are rejected.

- [ ] **Step 1: Create future iat tests**

```rust
#[test]
fn test_jwt_future_iat_rejected() {
    let config = auth::JwtConfig::new("test_secret".to_string());
    let now = Utc::now().timestamp();
    
    // Create JWT with iat in future (tomorrow)
    let future_iat = now + 86400;
    let claims = Claims {
        user_id: "user123".to_string(),
        sub: "user123".to_string(),
        iat: future_iat,
        exp: future_iat + 3600,
    };
    
    // Manually create token with future iat
    let token = create_jwt_with_claims(&claims, "test_secret");
    
    // Validation should reject it
    let result = config.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_iat_boundary_accepted() {
    // iat = now should be accepted
    // iat = now - 1 second should be accepted
}

#[test]
fn test_jwt_iat_way_in_future_rejected() {
    // iat = now + 1 year should be rejected
}
```

- [ ] **Step 2: Add iat validation to jwt_manager.rs**

Check `iat <= current_time + clock_skew` (e.g., 5 second skew).

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_jwt_future_iat`

- [ ] **Step 4: Commit**

`git commit -m "test: add JWT future iat validation tests"`

---

### Task 10: LogSmartAdmin Role Override Tests

**Files:**
- Create: `tests/security_critical_tests.rs`
- Modify: `back-end/src/handlers/user_handlers.rs` (lines 85-136)

**Description:** Verify company managers can't modify LogSmartAdmin users.

- [ ] **Step 1: Create role override tests**

```rust
#[tokio::test]
async fn test_company_manager_cannot_modify_logsmart_admin() {
    let pool = common::setup_test_db().await;
    
    let company = common::CompanyFactory::create_basic();
    
    let company_mgr = UserRecord {
        role: UserRole::CompanyManager,
        company_id: Some(company.id.clone()),
        ..common::UserFactory::create_basic()
    };
    
    let admin = UserRecord {
        role: UserRole::LogSmartAdmin,
        company_id: None,
        ..common::UserFactory::create_basic()
    };
    
    // Company manager tries to update LogSmartAdmin
    let result = handlers::user_handlers::admin_update_member_profile(
        &pool,
        &company_mgr,
        &admin.id,
        UpdateRequest { first_name: "Hacked".to_string() },
    ).await;
    
    // Should be forbidden
    assert!(result.is_err());
}

#[tokio::test]
async fn test_logsmart_admin_cannot_be_deleted_by_company_mgr() {
    // Company manager tries to delete LogSmartAdmin user
    // Should fail with authorization error
}
```

- [ ] **Step 2: Add mutual role comparison in authorization**

Check target role in addition to actor role.

- [ ] **Step 3: Run tests**

`cargo test --test security_critical_tests test_company_manager_cannot_modify`

- [ ] **Step 4: Commit**

`git commit -m "test: add LogSmartAdmin role protection tests"`

---

## TIER 2: BUSINESS LOGIC TESTS (30 tests)

### Task 11: Clock 24-Hour Capping Edge Cases

**Files:**
- Create: `tests/business_logic_tests.rs`
- Modify: `back-end/src/services/clock_service.rs` (lines 11-18)

**Description:** Test clock duration capping at exactly 24-hour boundary.

- [ ] **Step 1: Create boundary tests**

```rust
#[test]
fn test_clock_capping_exactly_24_hours() {
    let clock_in = Utc::now();
    let clock_out = clock_in + Duration::hours(24);
    
    let capped = clock_service::capped_clock_out_time(clock_in, clock_out);
    
    // Should be exactly at 24 hours (not 23:59:59)
    assert_eq!(capped, clock_in + Duration::hours(24));
}

#[test]
fn test_clock_capping_over_24_hours() {
    let clock_in = Utc::now();
    let clock_out = clock_in + Duration::hours(25);
    
    let capped = clock_service::capped_clock_out_time(clock_in, clock_out);
    
    // Should be capped to 24 hours, not 25
    assert_eq!(capped, clock_in + Duration::hours(24));
}

#[test]
fn test_clock_capping_under_24_hours() {
    let clock_in = Utc::now();
    let clock_out = clock_in + Duration::hours(12);
    
    let capped = clock_service::capped_clock_out_time(clock_in, clock_out);
    
    // Should remain unchanged
    assert_eq!(capped, clock_out);
}

#[test]
fn test_clock_capping_with_dst_transition() {
    // Test with DST transition during period
    // Duration calculation should still be correct
}
```

- [ ] **Step 2: Verify capping logic handles boundary correctly**

Check implementation for off-by-one errors.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_clock_capping`

- [ ] **Step 4: Commit**

`git commit -m "test: add clock 24-hour capping edge case tests"`

---

### Task 12: Clock Out When No Status Tests

**Files:**
- Create: `tests/business_logic_tests.rs`
- Modify: `back-end/src/services/clock_service.rs` (lines 63-106)

**Description:** Test clock out behavior when user is not clocked in.

- [ ] **Step 1: Create error handling tests**

```rust
#[tokio::test]
async fn test_clock_out_without_clock_in() {
    let pool = common::setup_test_db().await;
    let user = common::UserFactory::create_basic();
    
    // Try to clock out without clocking in
    let result = clock_service::clock_out(&pool, &user.id).await;
    
    // Should return error, not create orphaned record
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clock_out_after_concurrent_delete() {
    // User clocks out, but clock record deleted concurrently
    // Should handle gracefully
}

#[tokio::test]
async fn test_clock_out_database_connection_failure() {
    // Connection drops during clock out
    // Should not leave inconsistent state
}
```

- [ ] **Step 2: Verify error handling in clock_service.rs**

Check that errors are propagated correctly.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_clock_out_without`

- [ ] **Step 4: Commit**

`git commit -m "test: add clock out error handling tests"`

---

### Task 13: Concurrent Clock In Race Condition Tests

**Files:**
- Create: `tests/business_logic_tests.rs`
- Modify: `back-end/src/services/clock_service.rs` (lines 25-57)

**Description:** Test simultaneous clock in requests don't create duplicates.

- [ ] **Step 1: Create concurrent clock in tests**

```rust
#[tokio::test]
async fn test_concurrent_clock_in_same_user() {
    let pool = common::setup_test_db().await;
    let user = common::UserFactory::create_basic();
    
    // Send two clock in requests concurrently
    let clock_in1 = clock_service::clock_in(&pool, &user.id);
    let clock_in2 = clock_service::clock_in(&pool, &user.id);
    
    let (result1, result2) = tokio::join!(clock_in1, clock_in2);
    
    // One should succeed, one should fail (user already clocked in)
    let success_count = [result1, result2].iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 1);
    
    // Should have exactly 1 clock event
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM clock_events WHERE user_id = $1 AND clocked_out_at IS NULL"
    ).bind(&user.id).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_rapid_clock_in_out_cycles() {
    // Multiple clock in/out cycles rapidly
    // Should not accumulate duplicate events
}
```

- [ ] **Step 2: Use database constraints to prevent duplicates**

Add unique constraint on (user_id, clocked_out_at IS NULL).

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_concurrent_clock_in`

- [ ] **Step 4: Commit**

`git commit -m "test: add concurrent clock in race condition tests"`

---

### Task 14: Template XSS Validation Tests

**Files:**
- Create: `tests/business_logic_tests.rs`
- Modify: `back-end/src/services/template_service.rs` (lines 31-112)

**Description:** Test template fields reject XSS payloads.

- [ ] **Step 1: Create XSS injection tests**

```rust
#[test]
fn test_template_field_label_xss_rejection() {
    let xss_payloads = vec![
        r#""><script>alert('xss')</script>"#,
        r#"javascript:alert('xss')"#,
        r#"<img src=x onerror=alert('xss')>"#,
        r#"<svg/onload=alert('xss')>"#,
    ];
    
    for payload in xss_payloads {
        let field = TemplateField {
            label: payload.to_string(),
            ..create_test_field()
        };
        
        let result = template_service::validate_template_field(&field);
        assert!(result.is_err(), "Payload not rejected: {}", payload);
    }
}

#[test]
fn test_template_field_helptext_xss_rejection() {
    // Help text with XSS
}

#[test]
fn test_template_css_injection_rejection() {
    // Color field: red;cursor:pointer;width:1000px
    // Should reject multi-value CSS
}

#[test]
fn test_template_path_traversal_rejection() {
    // Template name: ../../../etc/passwd
    // Should reject or sanitize
}
```

- [ ] **Step 2: Add sanitization/validation to template fields**

Use allowlist for permitted field properties.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_template_field`

- [ ] **Step 4: Commit**

`git commit -m "test: add template XSS prevention tests"`

---

### Task 15: Template Field Coordinate Validation Tests

**Files:**
- Create: `tests/business_logic_tests.rs`

**Description:** Test template fields reject invalid coordinates.

- [ ] **Step 1: Create coordinate validation tests**

```rust
#[test]
fn test_template_field_negative_x_coordinate() {
    let field = TemplateField {
        x: -1,
        y: 100,
        ..create_test_field()
    };
    
    let result = template_service::validate_template_field(&field);
    assert!(result.is_err(), "Negative X coordinate should be rejected");
}

#[test]
fn test_template_field_zero_coordinates() {
    // x=0, y=0 should be valid (top-left)
}

#[test]
fn test_template_field_excessive_coordinates() {
    // x=100000, y=100000 should be rejected (out of canvas)
}
```

- [ ] **Step 2: Add bounds checking to template validation**

Check that x and y are within reasonable canvas size.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_template_field_coordinate`

- [ ] **Step 4: Commit**

`git commit -m "test: add template coordinate validation tests"`

---

### Task 16-20: Period Calculation Edge Cases (5 tests)

**Files:**
- Create: `tests/business_logic_tests.rs`

**Description:** Test period calculations for leap years, month boundaries, etc.

- [ ] **Step 1: Leap year edge case**

```rust
#[test]
fn test_period_leap_year_february_29() {
    // Period created on Feb 29 (leap year)
    // Next occurrence should be Feb 28 (non-leap year)
    let created = Utc.ymd_opt(2024, 2, 29).unwrap();
    let next = next_period_date(created, Frequency::Yearly);
    // Should be Feb 28 or Feb 29 (depending on policy)
}

#[test]
fn test_period_monthly_31st_in_28_day_month() {
    // Form due on 31st, but next month is Feb (28 days)
    // Should use day 28 for that month
}

#[test]
fn test_period_weekly_day_boundary() {
    // Weekly form on Sunday
    // Boundary conditions when week crosses month
}

#[test]
fn test_period_daily_dst_transition() {
    // Daily form during DST transition
    // Should still trigger correctly
}

#[test]
fn test_period_calculation_far_future() {
    // Calculate period 1 year in future
    // Should not panic or overflow
}
```

- [ ] **Step 2: Add period calculation tests**

Verify all edge cases in scheduling logic.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_period`

- [ ] **Step 4: Commit**

`git commit -m "test: add period calculation edge case tests"`

---

### Task 21: Concurrent Invitation Acceptance Race Condition Tests

**Files:**
- Create: `tests/business_logic_tests.rs`

**Description:** Test two users accepting same invitation simultaneously.

- [ ] **Step 1: Create concurrent acceptance tests**

```rust
#[tokio::test]
async fn test_concurrent_invitation_acceptance() {
    let pool = common::setup_test_db().await;
    let company = common::CompanyFactory::create_basic();
    let invitation = Invitation {
        id: "invite1".to_string(),
        company_id: company.id.clone(),
        email: "user@example.com".to_string(),
        token: "token123".to_string(),
        role: UserRole::Staff,
        branch_id: None,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        accepted_at: None,
        cancelled_at: None,
    };
    
    // Two concurrent acceptance attempts
    let accept1 = handlers::invitation_handlers::accept_invitation(
        &pool, &invitation.token, "Password123!"
    );
    let accept2 = handlers::invitation_handlers::accept_invitation(
        &pool, &invitation.token, "Password123!"
    );
    
    let (result1, result2) = tokio::join!(accept1, accept2);
    
    // One should succeed, one should fail (already accepted)
    let success_count = [result1, result2].iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 1);
}
```

- [ ] **Step 2: Add idempotency check to invitation acceptance**

Use database constraints or application logic.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_concurrent_invitation`

- [ ] **Step 4: Commit**

`git commit -m "test: add concurrent invitation acceptance tests"`

---

### Task 22: Invitation Expiry Boundary Tests

**Files:**
- Create: `tests/business_logic_tests.rs`

**Description:** Test invitation validation at exact expiry time.

- [ ] **Step 1: Create expiry boundary tests**

```rust
#[test]
fn test_invitation_expires_exactly_at_boundary() {
    let now = Utc::now();
    let invitation = Invitation {
        expires_at: now,
        ..create_test_invitation()
    };
    
    // At exact boundary - should be expired
    let is_valid = invitation.is_valid(now);
    assert!(!is_valid);
}

#[test]
fn test_invitation_valid_one_second_before() {
    let now = Utc::now();
    let invitation = Invitation {
        expires_at: now + Duration::seconds(1),
        ..create_test_invitation()
    };
    
    // Should still be valid 1 second before expiry
    let is_valid = invitation.is_valid(now);
    assert!(is_valid);
}

#[test]
fn test_invitation_invalid_one_second_after() {
    let now = Utc::now();
    let invitation = Invitation {
        expires_at: now - Duration::seconds(1),
        ..create_test_invitation()
    };
    
    // Should be expired 1 second after
    let is_valid = invitation.is_valid(now);
    assert!(!is_valid);
}
```

- [ ] **Step 2: Verify expiry comparison logic**

Check for `<=` vs `<` correctness.

- [ ] **Step 3: Run tests**

`cargo test --test business_logic_tests test_invitation_expires`

- [ ] **Step 4: Commit**

`git commit -m "test: add invitation expiry boundary tests"`

---

## TIER 3: DATA CONSISTENCY TESTS (10 tests)

### Task 23: Email Failure Transaction Tests

**Files:**
- Create: `tests/data_consistency_tests.rs`

**Description:** Test email service failures don't leave DB in inconsistent state.

- [ ] **Step 1: Create email failure tests**

```rust
#[tokio::test]
async fn test_registration_email_failure_rolls_back_user() {
    let pool = common::setup_test_db().await;
    
    // Mock email service to fail
    let email_svc = MockEmailService::new_failing();
    
    // Attempt registration
    let result = handlers::auth_handlers::register(
        &pool,
        &email_svc,
        &RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            company_name: "TestCo".to_string(),
            company_address: "123 Main".to_string(),
        },
    ).await;
    
    // Should fail
    assert!(result.is_err());
    
    // User should NOT exist in database
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    ).bind("test@example.com").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_password_reset_email_failure() {
    // Email sent but DB update fails - verify rollback
}

#[tokio::test]
async fn test_invitation_email_failure() {
    // Invitation created but email fails - verify no orphaned invitations
}
```

- [ ] **Step 2: Ensure email sending is atomic with DB operations**

Use database transactions properly.

- [ ] **Step 3: Run tests**

`cargo test --test data_consistency_tests test_registration_email_failure`

- [ ] **Step 4: Commit**

`git commit -m "test: add email transaction consistency tests"`

---

### Task 24-27: Cache & Race Condition Tests (4 tests)

**Files:**
- Create: `tests/data_consistency_tests.rs`

**Description:** Test cache invalidation and concurrent data modifications.

- [ ] **Step 1: Create cache invalidation tests**

```rust
#[tokio::test]
async fn test_cache_invalidation_on_user_update() {
    let pool = common::setup_test_db().await;
    let mut user = common::UserFactory::create_basic();
    
    // Cache user
    cache::insert(&user);
    
    // Update user in database
    user.first_name = "Updated".to_string();
    sqlx::query("UPDATE users SET first_name = $1 WHERE id = $2")
        .bind(&user.first_name)
        .bind(&user.id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Cache should be invalidated
    cache::invalidate(&user.id);
    
    // Next read should get updated data
    let cached = cache::get(&user.id);
    assert!(cached.is_none()); // Should not have stale data
}

#[tokio::test]
async fn test_concurrent_user_modifications() {
    // Two concurrent updates to same user
    // Should not lose updates (last write wins or conflict detected)
}

#[tokio::test]
async fn test_cross_database_sync_eventual_consistency() {
    // PostgreSQL and MongoDB consistency
}

#[tokio::test]
async fn test_profile_picture_upload_orphaned_files() {
    // Upload fails after MongoDB insert
    // File should be cleaned up
}
```

- [ ] **Step 2: Add cache invalidation logic**

Ensure cache is cleared on modifications.

- [ ] **Step 3: Run tests**

`cargo test --test data_consistency_tests test_cache_invalidation`

- [ ] **Step 4: Commit**

`git commit -m "test: add cache and concurrency consistency tests"`

---

## TIER 4: AUTHORIZATION TESTS (10 tests)

### Task 28: Cross-Company Data Access Tests

**Files:**
- Create: `tests/authorization_tests.rs`

**Description:** Verify users can't access other companies' data.

- [ ] **Step 1: Create cross-company access tests**

```rust
#[tokio::test]
async fn test_user_cannot_view_other_company_logs() {
    let pool = common::setup_test_db().await;
    
    let company1 = common::CompanyFactory::create_basic();
    let company2 = common::CompanyFactory::create_basic();
    
    let user1 = UserRecord {
        company_id: Some(company1.id.clone()),
        ..common::UserFactory::create_basic()
    };
    
    let user2 = UserRecord {
        company_id: Some(company2.id.clone()),
        ..common::UserFactory::create_basic()
    };
    
    // Create log for user2
    let log = LogEntry {
        user_id: user2.id.clone(),
        company_id: company2.id.clone(),
        ..create_test_log()
    };
    
    // User1 tries to access user2's log
    let result = handlers::log_handlers::get_log(&pool, &user1, &log.id).await;
    
    // Should be forbidden
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_cannot_list_other_company_users() {
    // Company1 user tries to list Company2 users
    // Should see empty list or forbidden error
}
```

- [ ] **Step 2: Verify all queries filter by company_id**

Check all endpoints enforce company isolation.

- [ ] **Step 3: Run tests**

`cargo test --test authorization_tests test_user_cannot_view_other`

- [ ] **Step 4: Commit**

`git commit -m "test: add cross-company access prevention tests"`

---

### Task 29-30: Rate Limit & Operations Tests (2 tests)

**Files:**
- Create: `tests/authorization_tests.rs`

**Description:** Test rate limiter cleanup and SMTP failures.

- [ ] **Step 1: Create rate limit cleanup tests**

```rust
#[tokio::test]
async fn test_rate_limit_cleanup_concurrent_access() {
    // While rate limiter cleanup runs, new requests come in
    // Should not crash or lose data
}

#[tokio::test]
async fn test_smtp_configuration_missing_graceful_failure() {
    // SMTP not configured
    // Should fail gracefully, not panic
    // User should see error message
}
```

- [ ] **Step 2: Add error handling for missing SMTP config**

Check configuration early, return clear error.

- [ ] **Step 3: Run tests**

`cargo test --test authorization_tests test_rate_limit_cleanup test_smtp`

- [ ] **Step 4: Commit**

`git commit -m "test: add rate limiter and SMTP tests"`

---

## SUMMARY & EXECUTION

**Total Tests to Create:** 84 tests across 4 files
**Estimated Hours:** 150-200 hours
**Timeline:** 4-5 weeks, 2 per week

**Files:**
1. `tests/security_critical_tests.rs` - 40 tests (1-2 weeks)
2. `tests/business_logic_tests.rs` - 30 tests (1-2 weeks)
3. `tests/data_consistency_tests.rs` - 10 tests (3-4 days)
4. `tests/authorization_tests.rs` - 10 tests (3-4 days)

**Commits:** 30 total (one per task)

**Execution:** Ready for parallel agent dispatch via `subagent-driven-development`
