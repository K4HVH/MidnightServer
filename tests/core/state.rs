use super::*;

fn test_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:50051".to_owned(),
        log_level: "info".to_owned(),
        log_style: "plain".to_owned(),
        cors_origins: vec!["*".to_owned()],
        database_url: "postgres://localhost/test".to_owned(),
        request_timeout_secs: 30,
    }
}

fn test_pool() -> PgPool {
    PgPool::connect_lazy("postgres://localhost/test").unwrap()
}

#[tokio::test]
async fn new_returns_arc() {
    let state = AppState::new(test_config(), test_pool());
    assert_eq!(Arc::strong_count(&state), 1);
}

#[tokio::test]
async fn config_returns_initial_values() {
    let state = AppState::new(test_config(), test_pool());
    let config = state.config();
    assert_eq!(config.listen_addr, "127.0.0.1:50051");
    assert_eq!(config.log_level, "info");
}

#[tokio::test]
async fn update_config_swaps_atomically() {
    let state = AppState::new(test_config(), test_pool());

    let new_config = Config {
        listen_addr: "0.0.0.0:9090".to_owned(),
        log_level: "debug".to_owned(),
        log_style: "json".to_owned(),
        cors_origins: vec!["http://example.com".to_owned()],
        database_url: "postgres://localhost/other".to_owned(),
        request_timeout_secs: 60,
    };

    state.update_config(new_config);

    let config = state.config();
    assert_eq!(config.listen_addr, "0.0.0.0:9090");
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.log_style, "json");
}

#[tokio::test]
async fn started_at_is_recent() {
    let before = Instant::now();
    let state = AppState::new(test_config(), test_pool());
    let after = Instant::now();

    assert!(state.started_at() >= before);
    assert!(state.started_at() <= after);
}

#[tokio::test]
async fn uptime_secs_is_zero_initially() {
    let state = AppState::new(test_config(), test_pool());
    assert_eq!(state.uptime_secs(), 0);
}

#[tokio::test]
async fn health_registry_accessible() {
    let state = AppState::new(test_config(), test_pool());
    let _health = state.health();
}

#[tokio::test]
async fn db_pool_accessible() {
    let state = AppState::new(test_config(), test_pool());
    let _db = state.db();
}

#[tokio::test]
async fn config_is_cloneable_via_load() {
    let state = AppState::new(test_config(), test_pool());
    let config1 = state.config();
    let config2 = state.config();
    assert_eq!(config1.listen_addr, config2.listen_addr);
}
