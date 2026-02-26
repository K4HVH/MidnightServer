#[derive(Debug, Clone)]
pub struct Config {
    pub listen_addr: String,
    pub log_level: String,
    pub log_style: String,
    pub cors_origins: Vec<String>,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen_addr: env_or("LISTEN_ADDR", "0.0.0.0:50051"),
            log_level: env_or("LOG_LEVEL", "info"),
            log_style: env_or("LOG_STYLE", "auto"),
            cors_origins: env_or("CORS_ORIGINS", "*")
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect(),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }

    pub fn cors_is_permissive(&self) -> bool {
        self.cors_origins.iter().any(|o| o == "*")
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}
