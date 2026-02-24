#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: String,
    pub log_level: String,
    pub cors_origins: Vec<String>,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
}

impl Config {
    /// Load from environment variables. See `.env.example` for available keys.
    pub fn from_env() -> Self {
        Self {
            listen_addr: env_or("LISTEN_ADDR", "0.0.0.0:50051"),
            log_level: env_or("LOG_LEVEL", "info"),
            cors_origins: env_or("CORS_ORIGINS", "*")
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect(),
            database_url: std::env::var("DATABASE_URL").ok(),
            redis_url: std::env::var("REDIS_URL").ok(),
        }
    }

    pub fn cors_is_permissive(&self) -> bool {
        self.cors_origins.iter().any(|o| o == "*")
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}
