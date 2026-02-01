//! Mock services for testing external dependencies

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Mock WebAuthn Service
#[derive(Debug, Clone)]
pub struct MockWebAuthnService {
    pub credentials: Arc<RwLock<HashMap<String, webauthn_rs::prelude::PublicKeyCredential>>>,
    pub challenges: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for MockWebAuthnService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWebAuthnService {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
            challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_credential(
        &self,
        user_id: &str,
        credential: webauthn_rs::prelude::PublicKeyCredential,
    ) {
        let mut credentials = self.credentials.write().await;
        credentials.insert(user_id.to_string(), credential);
    }

    pub async fn get_credential(
        &self,
        user_id: &str,
    ) -> Option<webauthn_rs::prelude::PublicKeyCredential> {
        let credentials = self.credentials.read().await;
        credentials.get(user_id).cloned()
    }

    pub async fn store_challenge(&self, challenge_id: &str, challenge: &str) {
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id.to_string(), challenge.to_string());
    }

    pub async fn get_challenge(&self, challenge_id: &str) -> Option<String> {
        let challenges = self.challenges.read().await;
        challenges.get(challenge_id).cloned()
    }

    pub async fn remove_challenge(&self, challenge_id: &str) {
        let mut challenges = self.challenges.write().await;
        challenges.remove(challenge_id);
    }
}

// Mock Email Service
#[derive(Debug, Clone)]
pub struct MockEmailService {
    pub sent_emails: Arc<RwLock<Vec<EmailMessage>>>,
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

impl Default for MockEmailService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEmailService {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            sent_emails: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Mocks sending an email.
    ///
    /// # Errors
    /// Always returns `Ok` in this mock implementation.
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let email = EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            sent_at: chrono::Utc::now(),
        };

        let mut sent_emails = self.sent_emails.write().await;
        sent_emails.push(email);

        Ok(())
    }

    pub async fn get_sent_emails(&self) -> Vec<EmailMessage> {
        let sent_emails = self.sent_emails.read().await;
        sent_emails.clone()
    }

    pub async fn clear_sent_emails(&self) {
        let mut sent_emails = self.sent_emails.write().await;
        sent_emails.clear();
    }

    pub async fn find_email_by_recipient(&self, recipient: &str) -> Vec<EmailMessage> {
        let sent_emails = self.sent_emails.read().await;
        sent_emails
            .iter()
            .filter(|email| email.to == recipient)
            .cloned()
            .collect()
    }

    pub async fn find_email_by_subject(&self, subject: &str) -> Vec<EmailMessage> {
        let sent_emails = self.sent_emails.read().await;
        sent_emails
            .iter()
            .filter(|email| email.subject.contains(subject))
            .cloned()
            .collect()
    }
}

#[async_trait]
pub trait EmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        self.send_email(to, subject, body).await
    }
}

// Mock OAuth Service
#[derive(Debug, Clone)]
pub struct MockOAuthService {
    pub tokens: Arc<RwLock<HashMap<String, OAuthToken>>>,
    pub user_info: Arc<RwLock<HashMap<String, OAuthUserInfo>>>,
}

#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub token_type: String,
}

#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub verified: bool,
}

impl Default for MockOAuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockOAuthService {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            user_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_token(&self, code: &str, token: OAuthToken) {
        let mut tokens = self.tokens.write().await;
        tokens.insert(code.to_string(), token);
    }

    pub async fn get_token(&self, code: &str) -> Option<OAuthToken> {
        let tokens = self.tokens.read().await;
        tokens.get(code).cloned()
    }

    pub async fn add_user_info(&self, access_token: &str, user_info: OAuthUserInfo) {
        let mut user_info_map = self.user_info.write().await;
        user_info_map.insert(access_token.to_string(), user_info);
    }

    pub async fn get_user_info(&self, access_token: &str) -> Option<OAuthUserInfo> {
        let user_info_map = self.user_info.read().await;
        user_info_map.get(access_token).cloned()
    }

    /// Mocks exchanging a code for an OAuth token.
    ///
    /// # Errors
    /// Returns an error if the code is not found in the mock store.
    pub async fn exchange_code_for_token(&self, code: &str) -> Result<OAuthToken> {
        match self.get_token(code).await {
            Some(token) => Ok(token),
            None => Err(anyhow::anyhow!("Invalid authorization code")),
        }
    }

    /// Mocks retrieving user info from an OAuth token.
    ///
    /// # Errors
    /// Returns an error if the token is not found in the mock store.
    pub async fn get_user_info_from_token(&self, access_token: &str) -> Result<OAuthUserInfo> {
        match self.get_user_info(access_token).await {
            Some(info) => Ok(info),
            None => Err(anyhow::anyhow!("Invalid access token")),
        }
    }
}

// Mock OAuth State Store
#[derive(Debug, Clone)]
pub struct MockOAuthStateStore {
    pub states: Arc<RwLock<HashMap<String, OAuthState>>>,
}

#[derive(Debug, Clone)]
pub struct OAuthState {
    pub state: String,
    pub redirect_uri: String,
    pub code_verifier: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Default for MockOAuthStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MockOAuthStateStore {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_state(&self, key: &str, state: OAuthState) {
        let mut states = self.states.write().await;
        states.insert(key.to_string(), state);
    }

    pub async fn get_state(&self, key: &str) -> Option<OAuthState> {
        let states = self.states.read().await;
        states.get(key).cloned()
    }

    pub async fn remove_state(&self, key: &str) {
        let mut states = self.states.write().await;
        states.remove(key);
    }

    pub async fn cleanup_expired(&self) {
        let mut states = self.states.write().await;
        let now = chrono::Utc::now();
        states.retain(|_, state| state.expires_at > now);
    }

    #[must_use] 
    pub fn generate_state_string(&self, length: usize) -> String {
        use rand::Rng;
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }
}

// Mock Rate Limiter
#[derive(Debug, Clone)]
pub struct MockRateLimiter {
    pub requests: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<chrono::Utc>>>>>,
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl MockRateLimiter {
    #[must_use] 
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = chrono::Utc::now();
        let window_start = now - chrono::Duration::seconds(self.window_seconds.cast_signed());

        let key_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        key_requests.retain(|&timestamp| timestamp > window_start);

        key_requests.len() < self.max_requests as usize
    }

    pub async fn record_request(&self, key: &str) {
        let mut requests = self.requests.write().await;
        let now = chrono::Utc::now();
        let key_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        key_requests.push(now);

        // Clean old requests
        let window_start = now - chrono::Duration::seconds(self.window_seconds.cast_signed());
        key_requests.retain(|&timestamp| timestamp > window_start);
    }

    pub async fn get_request_count(&self, key: &str) -> usize {
        let requests = self.requests.read().await;
        if let Some(key_requests) = requests.get(key) {
            let now = chrono::Utc::now();
            let window_start = now - chrono::Duration::seconds(self.window_seconds.cast_signed());
            key_requests
                .iter()
                .filter(|&&timestamp| timestamp > window_start)
                .count()
        } else {
            0
        }
    }

    pub async fn reset(&self) {
        let mut requests = self.requests.write().await;
        requests.clear();
    }
}

// Helper functions for creating mock instances
#[must_use] 
pub fn create_mock_webauthn() -> webauthn_rs::Webauthn {
    // This will need to be implemented based on the actual WebAuthnState structure
    // For now, return a placeholder
    todo!("Implement mock WebAuthnState creation")
}

#[must_use] 
pub fn create_mock_google_oauth() -> crate::services::GoogleOAuthClient {
    // This will need to be implemented based on the actual OAuthService structure
    // For now, return a placeholder
    todo!("Implement mock OAuthService creation")
}

#[must_use] 
pub fn create_mock_oauth_state_store() -> crate::handlers::OAuthStateStore {
    // This will need to be implemented based on the actual OAuthStateStore structure
    // For now, return a placeholder
    todo!("Implement mock OAuthStateStore creation")
}
