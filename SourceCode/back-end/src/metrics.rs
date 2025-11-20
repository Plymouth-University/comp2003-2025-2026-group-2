use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Clone)]
pub struct Metrics {
    pub total_requests: Arc<AtomicU64>,
    pub successful_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
    pub login_attempts: Arc<AtomicU64>,
    pub login_successes: Arc<AtomicU64>,
    pub login_failures: Arc<AtomicU64>,
    pub registrations: Arc<AtomicU64>,
    pub invitations_sent: Arc<AtomicU64>,
    pub invitations_accepted: Arc<AtomicU64>,
    pub rate_limit_hits: Arc<AtomicU64>,
}

impl Metrics {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            login_attempts: Arc::new(AtomicU64::new(0)),
            login_successes: Arc::new(AtomicU64::new(0)),
            login_failures: Arc::new(AtomicU64::new(0)),
            registrations: Arc::new(AtomicU64::new(0)),
            invitations_sent: Arc::new(AtomicU64::new(0)),
            invitations_accepted: Arc::new(AtomicU64::new(0)),
            rate_limit_hits: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_total_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_successful_requests(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed_requests(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_login_attempts(&self) {
        self.login_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_login_successes(&self) {
        self.login_successes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_login_failures(&self) {
        self.login_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_registrations(&self) {
        self.registrations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_invitations_sent(&self) {
        self.invitations_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_invitations_accepted(&self) {
        self.invitations_accepted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rate_limit_hits(&self) {
        self.rate_limit_hits.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use] 
    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            login_attempts: self.login_attempts.load(Ordering::Relaxed),
            login_successes: self.login_successes.load(Ordering::Relaxed),
            login_failures: self.login_failures.load(Ordering::Relaxed),
            registrations: self.registrations.load(Ordering::Relaxed),
            invitations_sent: self.invitations_sent.load(Ordering::Relaxed),
            invitations_accepted: self.invitations_accepted.load(Ordering::Relaxed),
            rate_limit_hits: self.rate_limit_hits.load(Ordering::Relaxed),
        }
    }

    pub fn spawn_logging_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let stats = self.get_stats();
                tracing::info!(
                    total_requests = stats.total_requests,
                    successful_requests = stats.successful_requests,
                    failed_requests = stats.failed_requests,
                    login_attempts = stats.login_attempts,
                    login_successes = stats.login_successes,
                    login_failures = stats.login_failures,
                    registrations = stats.registrations,
                    invitations_sent = stats.invitations_sent,
                    invitations_accepted = stats.invitations_accepted,
                    rate_limit_hits = stats.rate_limit_hits,
                    "Metrics snapshot"
                );
            }
        });
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub login_attempts: u64,
    pub login_successes: u64,
    pub login_failures: u64,
    pub registrations: u64,
    pub invitations_sent: u64,
    pub invitations_accepted: u64,
    pub rate_limit_hits: u64,
}

pub struct RequestTimer {
    start: Instant,
}

impl Default for RequestTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestTimer {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    #[must_use] 
    pub fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or_default()
    }
}
