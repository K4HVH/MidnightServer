use std::sync::Arc;
use std::time::Instant;

use arc_swap::{ArcSwap, Guard};
use sqlx::PgPool;

use super::config::Config;
use super::health::HealthRegistry;

#[allow(dead_code)]
pub struct AppState {
    config: ArcSwap<Config>,
    db: PgPool,
    health: HealthRegistry,
    started_at: Instant,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(config: Config, db: PgPool) -> Arc<Self> {
        Arc::new(Self {
            config: ArcSwap::from_pointee(config),
            db,
            health: HealthRegistry::new(),
            started_at: Instant::now(),
        })
    }

    pub fn config(&self) -> Guard<Arc<Config>> {
        self.config.load()
    }

    pub fn db(&self) -> &PgPool {
        &self.db
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

#[cfg(test)]
#[path = "../../tests/core/state.rs"]
mod tests;
