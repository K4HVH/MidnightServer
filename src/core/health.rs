use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub type HealthCheckFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Serving,
    NotServing,
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub id: Uuid,
    pub name: String,
    pub status: ServiceStatus,
    pub interval: Duration,
    pub registered_at: Instant,
    pub version: Option<String>,
    pub message: Option<String>,
}

impl ServiceHealth {
    pub fn uptime(&self) -> Duration {
        self.registered_at.elapsed()
    }
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct HealthRegistry {
    services: Arc<RwLock<HashMap<Uuid, ServiceHealth>>>,
    tasks: RwLock<HashMap<Uuid, JoinHandle<()>>>,
}

#[allow(dead_code)]
impl HealthRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(
        &self,
        name: impl Into<String>,
        interval: Duration,
        version: Option<String>,
        check: HealthCheckFn,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let name = name.into();

        let initial_probe = run_probe(&name, &check).await;
        let health = ServiceHealth {
            id,
            name: name.clone(),
            status: initial_probe.0,
            interval,
            registered_at: Instant::now(),
            version,
            message: initial_probe.1,
        };

        self.services.write().await.insert(id, health);

        let services = Arc::clone(&self.services);
        let probe_name = name.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let (status, message) = run_probe(&probe_name, &check).await;
                if let Some(svc) = services.write().await.get_mut(&id) {
                    svc.status = status;
                    svc.message = message;
                }
            }
        });

        self.tasks.write().await.insert(id, handle);

        tracing::info!(service = %name, %id, "health service registered");
        id
    }

    pub async fn deregister(&self, id: &Uuid) {
        if let Some(handle) = self.tasks.write().await.remove(id) {
            handle.abort();
        }
        self.services.write().await.remove(id);
    }

    pub async fn get(&self, id: &Uuid) -> Option<ServiceHealth> {
        self.services.read().await.get(id).cloned()
    }

    pub async fn get_by_name(&self, name: &str) -> Option<ServiceHealth> {
        self.services
            .read()
            .await
            .values()
            .find(|h| h.name == name)
            .cloned()
    }

    pub async fn list(&self) -> Vec<ServiceHealth> {
        self.services.read().await.values().cloned().collect()
    }
}

async fn run_probe(name: &str, check: &HealthCheckFn) -> (ServiceStatus, Option<String>) {
    match tokio::time::timeout(PROBE_TIMEOUT, check()).await {
        Ok(Ok(())) => (ServiceStatus::Serving, None),
        Ok(Err(err)) => {
            tracing::warn!(service = %name, %err, "health probe failed");
            (ServiceStatus::NotServing, Some(err))
        }
        Err(_) => {
            tracing::warn!(service = %name, "health probe timed out");
            (
                ServiceStatus::NotServing,
                Some("probe timed out".to_owned()),
            )
        }
    }
}

impl Drop for HealthRegistry {
    fn drop(&mut self) {
        if let Ok(tasks) = self.tasks.try_write() {
            for handle in tasks.values() {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/core/health.rs"]
mod tests;
