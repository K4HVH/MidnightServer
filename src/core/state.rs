use std::sync::Arc;
use std::time::Instant;

use arc_swap::{ArcSwap, Guard};

use super::config::Config;
use super::health::HealthRegistry;

pub struct AppState {
    config: ArcSwap<Config>,
    health: HealthRegistry,
    started_at: Instant,
}

impl AppState {
    pub fn new(config: Config) -> Arc<Self> {
        Arc::new(Self {
            config: ArcSwap::from_pointee(config),
            health: HealthRegistry::new(),
            started_at: Instant::now(),
        })
    }

    pub fn config(&self) -> Guard<Arc<Config>> {
        self.config.load()
    }

    pub fn health(&self) -> &HealthRegistry {
        &self.health
    }

    pub fn update_config(&self, new_config: Config) {
        self.config.store(Arc::new(new_config));
        tracing::info!("configuration updated at runtime");
    }

    pub fn reload_config_from_env(&self) {
        self.update_config(Config::from_env());
    }

    pub fn started_at(&self) -> Instant {
        self.started_at
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}
