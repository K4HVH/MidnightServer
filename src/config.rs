/// Server configuration, loaded from environment variables.
pub struct Config {
    /// Address to bind the server to (e.g. "0.0.0.0:50051").
    pub listen_addr: String,
}

impl Config {
    /// Reads configuration from environment variables with sensible defaults.
    ///
    /// | Variable      | Default          | Description                |
    /// |---------------|------------------|----------------------------|
    /// | `LISTEN_ADDR` | `0.0.0.0:50051`  | Server bind address        |
    pub fn from_env() -> Self {
        Self {
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:50051".into()),
        }
    }
}
