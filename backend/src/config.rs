
use std::env;

pub struct AppConfig {
    pub greptime_url: String,
    pub database: String,
    pub host: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            greptime_url: env::var("GREPTIME_URL")
                .unwrap_or_else(|_| "http://localhost:4000".to_string()),

            database: env::var("GREPTIME_DATABASE")
                .unwrap_or_else(|_| "public".to_string()),

            host: env::var("HOST_NAME")
                .unwrap_or_else(|_| "macbook-air".to_string()),
        }
    }
}
