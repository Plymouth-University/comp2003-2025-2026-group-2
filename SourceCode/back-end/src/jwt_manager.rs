use crate::auth::JwtConfig;
use std::sync::OnceLock;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub struct JwtManager;

impl JwtManager {
    pub fn init(secret: String) {
        let _ = JWT_SECRET.set(secret);
    }

    pub fn get_config() -> JwtConfig {
        let secret = JWT_SECRET
            .get_or_init(|| {
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "logsmart_secret_key_for_testing".to_string())
            })
            .clone();
        JwtConfig::new(secret)
    }

    pub fn get_secret() -> String {
        JWT_SECRET
            .get_or_init(|| {
                std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "logsmart_secret_key_for_testing".to_string())
            })
            .clone()
    }
}
