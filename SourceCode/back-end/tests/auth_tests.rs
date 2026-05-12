use back_end::auth::{
    Claims, JwtConfig, generate_uuid6_token, hash_password, validate_email,
    validate_password_policy, verify_password,
};
use uuid::Uuid;

// ===== JWT Config Tests =====

#[test]
fn test_jwt_config_new() {
    let config = JwtConfig::new("test_secret".to_string());
    assert_eq!(config.secret, "test_secret");
}

#[test]
fn test_jwt_config_empty_secret() {
    let config = JwtConfig::new("".to_string());
    assert_eq!(config.secret, "");
}

#[test]
fn test_jwt_config_with_unicode_secret() {
    let unicode_secret = "sëcrët_wïth_ünïcödé_çhàracters".to_string();
    let config = JwtConfig::new(unicode_secret);
    let result = config.generate_token("user123", 1);
    assert!(result.is_ok());
}

// ===== Token Generation Tests =====

#[test]
fn test_generate_token_success() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123", 1);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(!token.is_empty());
}

#[test]
fn test_generate_token_different_user_ids() {
    let config = JwtConfig::new("test_secret".to_string());
    let token1 = config.generate_token("user1", 1).unwrap();
    let token2 = config.generate_token("user2", 1).unwrap();
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_token_with_different_expiry() {
    let config = JwtConfig::new("test_secret".to_string());
    let token1 = config.generate_token("user123", 1).unwrap();
    let token2 = config.generate_token("user123", 24).unwrap();
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_token_with_special_characters() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user@domain.com+tag#123".to_string();
    let result = config.generate_token(user_id.as_str(), 1);
    assert!(result.is_ok());
}

// ===== Token Validation Tests =====

#[test]
fn test_validate_token_success() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123".to_string();
    let token = config.generate_token(&user_id, 24).unwrap();
    let claims = config.validate_token(&token);
    assert!(claims.is_ok());
    let claims = claims.unwrap();
    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
}

#[test]
fn test_validate_token_success_with_timestamps() {
    let config = JwtConfig::new("test_secret".to_string());
    let user_id = "user123".to_string();
    let token = config.generate_token(user_id.as_str(), 24).unwrap();
    let result = config.validate_token(&token);
    assert!(result.is_ok());
    let claims = result.unwrap();
    assert_eq!(claims.user_id, user_id);
    assert_eq!(claims.sub, user_id);
    assert!(claims.exp > chrono::Utc::now().timestamp());
    assert!(claims.iat <= chrono::Utc::now().timestamp());
}

#[test]
fn test_validate_token_invalid_token() {
    let config = JwtConfig::new("test_secret".to_string());
    let result = config.validate_token("invalid_token");
    assert!(result.is_err());
}

#[test]
fn test_validate_token_empty_token() {
    let config = JwtConfig::new("test_secret".to_string());
    let result = config.validate_token("");
    assert!(result.is_err());
}

#[test]
fn test_validate_token_with_wrong_secret() {
    let config1 = JwtConfig::new("secret1".to_string());
    let config2 = JwtConfig::new("secret2".to_string());
    let token = config1.generate_token("user123", 24).unwrap();
    let result = config2.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_validate_token_expired() {
    let config = JwtConfig::new("test_secret".to_string());
    let token = config.generate_token("user123", -1).unwrap();
    let result = config.validate_token(&token);
    assert!(result.is_err());
}

// ===== Password Hashing Tests =====

#[test]
fn test_hash_password_success() {
    let password = "mypassword123";
    let hash = hash_password(password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_password_consistent() {
    let password = "mypassword123";
    let hash1 = hash_password(password).unwrap();
    let hash2 = hash_password(password).unwrap();
    assert_ne!(hash1, hash2);
    assert!(verify_password(password, &hash1).unwrap());
    assert!(verify_password(password, &hash2).unwrap());
}

#[test]
fn test_hash_password_different_inputs() {
    let password1 = "password123";
    let password2 = "password456";
    let hash1 = hash_password(password1).unwrap();
    let hash2 = hash_password(password2).unwrap();
    assert_ne!(hash1, hash2);
}

#[test]
fn test_hash_password_empty_string() {
    let password = "";
    let hash = hash_password(password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_password_long_password() {
    let password = "a".repeat(1000);
    let hash = hash_password(&password);
    assert!(hash.is_ok());
    let hash = hash.unwrap();
    assert!(!hash.is_empty());
}

#[test]
fn test_hash_password_with_unicode() {
    let password = "Pásswörd123!";
    let hash = hash_password(password);
    assert!(hash.is_ok());
}

#[test]
fn test_hash_password_with_emoji() {
    let password = "Password🔒123!";
    let hash = hash_password(password).unwrap();
    assert!(!hash.is_empty());
}

// ===== Password Verification Tests =====

#[test]
fn test_verify_password_correct() {
    let password = "mypassword123";
    let hash = hash_password(password).unwrap();
    let result = verify_password(password, &hash);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_password_incorrect() {
    let password = "mypassword123";
    let hash = hash_password(password).unwrap();
    let result = verify_password("wrongpassword", &hash);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_verify_password_wrong_password() {
    let hash = hash_password("correct_password").unwrap();
    let result = verify_password("wrong_password", &hash);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_verify_password_empty_hash() {
    let password = "mypassword123";
    let result = verify_password(password, "invalid_hash");
    assert!(result.is_err());
}

#[test]
fn test_verify_password_invalid_hash() {
    let result = verify_password("password", "invalid_hash");
    assert!(result.is_err());
}

#[test]
fn test_verify_password_empty_password_with_hash() {
    let hash = hash_password("").unwrap();
    let result = verify_password("", &hash);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_verify_password_emoji() {
    let emoji_password = "Password🔒123!";
    let hash = hash_password(emoji_password).unwrap();
    let verify_result = verify_password(emoji_password, &hash);
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap());
}

// ===== Claims Structure Tests =====

#[test]
fn test_claims_struct() {
    let claims = Claims {
        sub: "user123".to_string(),
        exp: 1234567890,
        iat: 1234567800,
        user_id: "user123".to_string(),
    };
    assert_eq!(claims.sub, "user123");
    assert_eq!(claims.user_id, "user123");
    assert_eq!(claims.exp, 1234567890);
}

// ===== Email Validation Tests =====

#[test]
fn test_validate_email_valid() {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    assert!(validate_email(&format!("user{}@example.com", unique_id)).is_ok());
    assert!(validate_email("test.user+tag@domain.co.uk").is_ok());
    assert!(validate_email("user123@test-domain.com").is_ok());
}

#[test]
fn test_validate_email_valid_emails() {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let valid_emails = vec![
        format!("test{}@example.com", unique_id),
        "user.name@domain.co.uk".to_string(),
        "user+tag@example.org".to_string(),
        "user_name123@test-domain.com".to_string(),
        "a@b.co".to_string(),
    ];

    for email in valid_emails {
        let result = validate_email(&email);
        assert!(result.is_ok(), "Email {} should be valid", email);
    }
}

#[test]
fn test_validate_email_invalid() {
    assert!(validate_email("").is_err());
    assert!(validate_email("invalid").is_err());
    assert!(validate_email("@example.com").is_err());
    assert!(validate_email("user@").is_err());
    assert!(validate_email("user name@example.com").is_err());
}

#[test]
fn test_validate_email_invalid_emails() {
    let invalid_emails = vec![
        "",
        "plainaddress",
        "@missingdomain.com",
        "missing@.com",
        "missing@domain",
        "spaces @domain.com",
        "user@domain .com",
        "user@domain@domain.com",
        "user@.domain.com",
        ".user@domain.com",
        "user.@domain.com",
        "user@domain..com",
    ];

    for email in invalid_emails {
        let result = validate_email(email);
        assert!(result.is_err(), "Email {} should be invalid", email);
    }
}

#[test]
fn test_validate_email_case_sensitivity() {
    let result = validate_email("Test@EXAMPLE.COM");
    assert!(result.is_ok());
}

#[test]
fn test_validate_email_too_long() {
    let long_email = format!("{}@example.com", "a".repeat(250));
    assert!(validate_email(&long_email).is_err());
}

#[test]
fn test_validate_email_exactly_max_length() {
    let user = "a".repeat(64);
    let domain = "a".repeat(250 - 64 - 1 - 4);
    let email = format!("{}@{}.com", user, domain);
    let result = validate_email(&email);
    assert!(result.is_ok());
}

#[test]
fn test_validate_email_with_international_domain() {
    let international_emails = vec!["test@xn--example.com"];
    for email in international_emails {
        let result = validate_email(email);
        // Just verify it doesn't panic
        let _ = result;
    }
}

// ===== Password Policy Validation Tests =====

#[test]
fn test_validate_password_policy_valid() {
    assert!(validate_password_policy("Password123!").is_ok());
    assert!(validate_password_policy("Secure@Pass1").is_ok());
    assert!(validate_password_policy("MyP@ssw0rd").is_ok());
}

#[test]
fn test_validate_password_policy_valid_passwords() {
    let valid_passwords = vec![
        "Password123!",
        "MySecureP@ssw0rd",
        "Complex_P@ssw0rd",
        "aB1@cdEF",
        "VerySecurePassword123!",
    ];

    for password in valid_passwords {
        let result = validate_password_policy(password);
        assert!(result.is_ok(), "Password {} should be valid", password);
    }
}

#[test]
fn test_validate_password_policy_too_short() {
    assert!(validate_password_policy("Pass1!").is_err());
    let short_passwords = vec!["", "a", "ab", "abc", "Abc1!", "Pass1"];
    for password in short_passwords {
        let result = validate_password_policy(password);
        assert!(result.is_err(), "Password {} should be too short", password);
    }
}

#[test]
fn test_validate_password_policy_exactly_min_length() {
    let min_password = "Abcdef1!";
    let result = validate_password_policy(min_password);
    assert!(result.is_ok());
}

#[test]
fn test_validate_password_policy_too_long() {
    let long_password = format!("Password1!{}", "a".repeat(200));
    assert!(validate_password_policy(&long_password).is_err());
    let very_long = "a".repeat(129);
    assert!(validate_password_policy(&very_long).is_err());
}

#[test]
fn test_validate_password_policy_exactly_max_length() {
    let max_password = format!("A{}1!", "b".repeat(125));
    let result = validate_password_policy(&max_password);
    assert!(result.is_ok());
}

#[test]
fn test_validate_password_policy_no_uppercase() {
    assert!(validate_password_policy("password123!").is_err());
    let passwords = vec!["password123!", "lowercase1!", "alllowercase"];
    for password in passwords {
        let result = validate_password_policy(password);
        assert!(
            result.is_err(),
            "Password {} should fail (no uppercase)",
            password
        );
    }
}

#[test]
fn test_validate_password_policy_no_lowercase() {
    assert!(validate_password_policy("PASSWORD123!").is_err());
    let passwords = vec!["PASSWORD123!", "UPPERCASE1!", "ALLUPPERCASE"];
    for password in passwords {
        let result = validate_password_policy(password);
        assert!(
            result.is_err(),
            "Password {} should fail (no lowercase)",
            password
        );
    }
}

#[test]
fn test_validate_password_policy_no_digit() {
    assert!(validate_password_policy("Password!").is_err());
    let passwords = vec!["Password!", "NoDigitsHere!", "allletters"];
    for password in passwords {
        let result = validate_password_policy(password);
        assert!(
            result.is_err(),
            "Password {} should fail (no digit)",
            password
        );
    }
}

#[test]
fn test_validate_password_policy_no_special() {
    assert!(validate_password_policy("Password123").is_err());
    let passwords = vec!["Password123", "NoSpecialChars123", "allalphanumeric"];
    for password in passwords {
        let result = validate_password_policy(password);
        assert!(
            result.is_err(),
            "Password {} should fail (no special)",
            password
        );
    }
}

#[test]
fn test_validate_password_policy_unicode_characters() {
    let unicode_password = "Pásswörd123!";
    let result = validate_password_policy(unicode_password);
    assert!(result.is_ok());
}

#[test]
fn test_validate_password_policy_with_emoji() {
    let emoji_password = "Password🔒123!";
    let result = validate_password_policy(emoji_password);
    assert!(result.is_ok());
}

// ===== UUID6 Token Generation Tests =====

#[test]
fn test_generate_invitation_token_format() {
    let token = generate_uuid6_token();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36);
    assert_eq!(token.chars().filter(|&c| c == '-').count(), 4);
}

#[test]
fn test_generate_uuid6_token_format() {
    let token = generate_uuid6_token();
    assert!(!token.is_empty());

    let parts: Vec<&str> = token.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert!(parts[4].len() >= 12);
}

#[test]
fn test_generate_invitation_token_unique() {
    let token1 = generate_uuid6_token();
    let token2 = generate_uuid6_token();
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_uuid6_token_uniqueness() {
    let token1 = generate_uuid6_token();
    let token2 = generate_uuid6_token();
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_uuid6_token_consistent_length() {
    let token1 = generate_uuid6_token();
    let token2 = generate_uuid6_token();
    assert_eq!(token1.len(), token2.len());
}

// ===== Integration Tests =====

#[test]
fn test_authentication_workflow() {
    let config = JwtConfig::new("test_secret".to_string());
    let password = "SecurePassword123!";

    assert!(validate_password_policy(password).is_ok());

    let hash = hash_password(password).unwrap();
    assert!(verify_password(password, &hash).unwrap());

    let user_id = "test_user".to_string();
    let token = config.generate_token(user_id.as_str(), 1).unwrap();
    let claims = config.validate_token(&token).unwrap();
    assert_eq!(claims.user_id, user_id);
}

#[test]
fn test_email_and_password_validation_combination() {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let valid_combinations = vec![
        (format!("user{}@example.com", unique_id), "Password123!"),
        ("test.user+tag@domain.co.uk".to_string(), "SecureP@ssw0rd"),
    ];

    for (email, password) in valid_combinations {
        assert!(
            validate_email(&email).is_ok(),
            "Email {} should be valid",
            email
        );
        assert!(
            validate_password_policy(password).is_ok(),
            "Password {} should be valid",
            password
        );
    }
}
