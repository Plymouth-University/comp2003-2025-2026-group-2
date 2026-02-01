use back_end::rate_limit::RateLimitState;
use std::net::{IpAddr, Ipv4Addr};
use std::thread;

#[test]
fn test_rate_limit_concurrency() {
    // Create a rate limit state
    let rate_limit = RateLimitState::new();

    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let mut handles = vec![];

    // Spawn 10 threads, each checking rate limit 100 times
    for _ in 0..10 {
        let rl = rate_limit.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Just check if it panics or deadlocks
                // We don't assert the result because with concurrency it's hard to predict
                // exact allow/deny without precise timing control
                let _ = rl.check_general(ip);
                let _ = rl.check_login(ip);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
