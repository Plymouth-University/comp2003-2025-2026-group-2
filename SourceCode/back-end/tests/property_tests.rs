use back_end::auth::{validate_email, validate_password_policy};
use proptest::prelude::*;

proptest! {
    // Test that email validation never panics for any input string
    #[test]
    fn test_email_validation_does_not_panic(email in "\\PC*") {
        let _ = validate_email(&email);
    }

    // Test that constructing a valid email structure usually passes validation
    #[test]
    fn test_valid_email_structure(
        user in "[a-zA-Z0-9_+-]+",
        domain in "[a-zA-Z0-9-]+",
        tld in "[a-zA-Z]{2,6}"
    ) {
        let email = format!("{}@{}.{}", user, domain, tld);

        // We only assert if it fits within length limits
        if email.len() <= 254 {
             let result = validate_email(&email);
             prop_assert!(result.is_ok(), "Email {} failed validation: {:?}", email, result.err());
        }
    }

    // Test password policy validation
    #[test]
    fn test_password_policy_validation(password in "\\PC*") {
        let result = validate_password_policy(&password);

        // Check length constraints
        if password.len() < 8 {
            prop_assert!(result.is_err(), "Short password {} should fail", password);
        } else if password.len() > 128 {
            prop_assert!(result.is_err(), "Long password should fail");
        } else {
            // If length is OK, check character requirements
            let has_uppercase = password.chars().any(char::is_uppercase);
            let has_lowercase = password.chars().any(char::is_lowercase);
            let has_digit = password.chars().any(char::is_numeric);
            let has_special = password.chars().any(|c| !c.is_alphanumeric());

            if !has_uppercase || !has_lowercase || !has_digit || !has_special {
                prop_assert!(result.is_err(), "Password {} missing requirements should fail", password);
            } else {
                // If all requirements met, it SHOULD pass
                prop_assert!(result.is_ok(), "Valid password {} should pass", password);
            }
        }
    }
}
