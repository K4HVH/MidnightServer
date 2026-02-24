use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub type HealthCheckFn = Box<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Serving,
    NotServing,
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub status: ServiceStatus,
    pub message: Option<String>,
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct HealthRegistry {
    services: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    tasks: RwLock<HashMap<String, JoinHandle<()>>>,
}

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
        check: HealthCheckFn,
    ) {
        let name = name.into();

        let initial = run_probe(&name, &check).await;
        self.services
            .write()
            .await
            .insert(name.clone(), initial);

        let services = Arc::clone(&self.services);
        let probe_name = name.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // skip the immediate first tick
            loop {
                ticker.tick().await;
                let health = run_probe(&probe_name, &check).await;
                services.write().await.insert(probe_name.clone(), health);
            }
        });

        if let Some(old) = self.tasks.write().await.insert(name, handle) {
            old.abort();
        }
    }

    pub async fn deregister(&self, name: &str) {
        if let Some(handle) = self.tasks.write().await.remove(name) {
            handle.abort();
        }
        self.services.write().await.remove(name);
    }

    pub async fn check(&self, service: &str) -> Option<ServiceHealth> {
        if service.is_empty() {
            return Some(self.aggregate().await);
        }
        self.services.read().await.get(service).cloned()
    }

    pub async fn list(&self) -> Vec<(String, ServiceHealth)> {
        self.services
            .read()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    async fn aggregate(&self) -> ServiceHealth {
        let services = self.services.read().await;
        let unhealthy: Vec<_> = services
            .iter()
            .filter(|(_, h)| h.status == ServiceStatus::NotServing)
            .map(|(name, h)| {
                match &h.message {
                    Some(msg) => format!("{name}: {msg}"),
                    None => name.clone(),
                }
            })
            .collect();

        if unhealthy.is_empty() {
            ServiceHealth {
                status: ServiceStatus::Serving,
                message: None,
            }
        } else {
            ServiceHealth {
                status: ServiceStatus::NotServing,
                message: Some(unhealthy.join("; ")),
            }
        }
    }
}

async fn run_probe(name: &str, check: &HealthCheckFn) -> ServiceHealth {
    match tokio::time::timeout(PROBE_TIMEOUT, check()).await {
        Ok(Ok(())) => ServiceHealth {
            status: ServiceStatus::Serving,
            message: None,
        },
        Ok(Err(err)) => {
            tracing::warn!(service = %name, %err, "health probe failed");
            ServiceHealth {
                status: ServiceStatus::NotServing,
                message: Some(err),
            }
        }
        Err(_) => {
            tracing::warn!(service = %name, "health probe timed out");
            ServiceHealth {
                status: ServiceStatus::NotServing,
                message: Some("probe timed out".to_owned()),
            }
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
