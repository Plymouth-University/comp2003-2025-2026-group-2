use back_end::auth::{hash_password, verify_password};
use std::time::Instant;

#[test]
fn benchmark_password_hashing() {
    let password = "my_secure_password";

    let start = Instant::now();
    let hash = hash_password(password).unwrap();
    let duration = start.elapsed();

    // Use println to show output if running with --nocapture
    println!("Password hashing took: {:?}", duration);

    // Argon2 should be intentionally slow, but not too slow for UX
    // Assuming default params (m=19456, t=2, p=1) it should be around 50-500ms depending on CPU
    assert!(
        duration.as_millis() > 5,
        "Hashing was surprisingly fast ({:?})",
        duration
    );
    assert!(
        duration.as_millis() < 2000,
        "Hashing was too slow ({:?})",
        duration
    );

    let start_verify = Instant::now();
    let valid = verify_password(password, &hash).unwrap();
    let verify_duration = start_verify.elapsed();

    println!("Password verification took: {:?}", verify_duration);
    assert!(valid);
    assert!(verify_duration.as_millis() < 2000, "Verification too slow");
}
