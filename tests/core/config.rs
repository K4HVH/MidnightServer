use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env(vars: &[(&str, &str)], f: impl FnOnce() + std::panic::UnwindSafe) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let all_keys = [
        "LISTEN_ADDR",
        "LOG_LEVEL",
        "LOG_STYLE",
        "CORS_ORIGINS",
        "DATABASE_URL",
        "REQUEST_TIMEOUT_SECS",
    ];
    unsafe {
        for key in &all_keys {
            std::env::remove_var(key);
        }
        for (key, value) in vars {
            std::env::set_var(key, value);
        }
    }
    let result = std::panic::catch_unwind(f);
    unsafe {
        for key in &all_keys {
            std::env::remove_var(key);
        }
    }
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn defaults_applied_when_env_unset() {
    with_env(&[("DATABASE_URL", "postgres://localhost/test")], || {
        let config = Config::from_env();
        assert_eq!(config.listen_addr, "0.0.0.0:50051");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_style, "auto");
        assert_eq!(config.cors_origins, vec!["*"]);
        assert_eq!(config.request_timeout_secs, 30);
    });
}

#[test]
fn env_vars_override_defaults() {
    with_env(
        &[
            ("LISTEN_ADDR", "127.0.0.1:9090"),
            ("LOG_LEVEL", "debug"),
            ("LOG_STYLE", "json"),
            ("CORS_ORIGINS", "http://a.com,http://b.com"),
            ("DATABASE_URL", "postgres://localhost/test"),
            ("REQUEST_TIMEOUT_SECS", "10"),
        ],
        || {
            let config = Config::from_env();
            assert_eq!(config.listen_addr, "127.0.0.1:9090");
            assert_eq!(config.log_level, "debug");
            assert_eq!(config.log_style, "json");
            assert_eq!(config.cors_origins, vec!["http://a.com", "http://b.com"]);
            assert_eq!(config.request_timeout_secs, 10);
        },
    );
}

#[test]
#[should_panic(expected = "DATABASE_URL must be set")]
fn panics_without_database_url() {
    with_env(&[], || {
        Config::from_env();
    });
}

#[test]
fn cors_is_permissive_with_wildcard() {
    with_env(&[("DATABASE_URL", "postgres://localhost/test")], || {
        let config = Config::from_env();
        assert!(config.cors_is_permissive());
    });
}

#[test]
fn cors_is_not_permissive_with_specific_origins() {
    with_env(
        &[
            ("CORS_ORIGINS", "http://localhost:3000"),
            ("DATABASE_URL", "postgres://localhost/test"),
        ],
        || {
            let config = Config::from_env();
            assert!(!config.cors_is_permissive());
        },
    );
}

#[test]
fn cors_origins_trimmed() {
    with_env(
        &[
            ("CORS_ORIGINS", " http://a.com , http://b.com "),
            ("DATABASE_URL", "postgres://localhost/test"),
        ],
        || {
            let config = Config::from_env();
            assert_eq!(config.cors_origins, vec!["http://a.com", "http://b.com"]);
        },
    );
}

#[test]
fn env_or_returns_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("__TEST_KEY_NONEXISTENT") };
    assert_eq!(env_or("__TEST_KEY_NONEXISTENT", "fallback"), "fallback");
}

#[test]
fn env_or_returns_env_value() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::set_var("__TEST_KEY_EXISTS", "from_env");
    }
    assert_eq!(env_or("__TEST_KEY_EXISTS", "fallback"), "from_env");
    unsafe { std::env::remove_var("__TEST_KEY_EXISTS") };
}
