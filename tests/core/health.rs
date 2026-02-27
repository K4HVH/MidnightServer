use super::*;

fn ok_check() -> HealthCheckFn {
    Box::new(|| Box::pin(async { Ok(()) }))
}

fn failing_check(msg: &str) -> HealthCheckFn {
    let msg = msg.to_owned();
    Box::new(move || {
        let msg = msg.clone();
        Box::pin(async move { Err(msg) })
    })
}

fn slow_check(duration: Duration) -> HealthCheckFn {
    Box::new(move || {
        Box::pin(async move {
            tokio::time::sleep(duration).await;
            Ok(())
        })
    })
}

#[tokio::test]
async fn register_returns_unique_ids() {
    let registry = HealthRegistry::new();
    let id1 = registry
        .register("svc1", Duration::from_secs(60), None, ok_check())
        .await;
    let id2 = registry
        .register("svc2", Duration::from_secs(60), None, ok_check())
        .await;
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn register_stores_service_metadata() {
    let registry = HealthRegistry::new();
    let id = registry
        .register(
            "myservice",
            Duration::from_secs(42),
            Some("1.2.3".to_owned()),
            ok_check(),
        )
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.name, "myservice");
    assert_eq!(health.interval, Duration::from_secs(42));
    assert_eq!(health.version.as_deref(), Some("1.2.3"));
    assert_eq!(health.id, id);
}

#[tokio::test]
async fn healthy_probe_sets_serving_status() {
    let registry = HealthRegistry::new();
    let id = registry
        .register("svc", Duration::from_secs(60), None, ok_check())
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::Serving);
    assert!(health.message.is_none());
}

#[tokio::test]
async fn failing_probe_sets_not_serving_with_message() {
    let registry = HealthRegistry::new();
    let id = registry
        .register(
            "bad",
            Duration::from_secs(60),
            None,
            failing_check("connection refused"),
        )
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::NotServing);
    assert_eq!(health.message.as_deref(), Some("connection refused"));
}

#[tokio::test]
async fn timeout_probe_sets_not_serving() {
    let registry = HealthRegistry::new();
    let id = registry
        .register(
            "slow",
            Duration::from_secs(60),
            None,
            slow_check(Duration::from_secs(10)),
        )
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::NotServing);
    assert_eq!(health.message.as_deref(), Some("probe timed out"));
}

#[tokio::test]
async fn version_is_optional() {
    let registry = HealthRegistry::new();
    let id = registry
        .register("svc", Duration::from_secs(60), None, ok_check())
        .await;

    let health = registry.get(&id).await.unwrap();
    assert!(health.version.is_none());
}

#[tokio::test]
async fn deregister_removes_service() {
    let registry = HealthRegistry::new();
    let id = registry
        .register("svc", Duration::from_secs(60), None, ok_check())
        .await;

    assert!(registry.get(&id).await.is_some());
    registry.deregister(&id).await;
    assert!(registry.get(&id).await.is_none());
}

#[tokio::test]
async fn deregister_nonexistent_does_not_panic() {
    let registry = HealthRegistry::new();
    let fake_id = Uuid::new_v4();
    registry.deregister(&fake_id).await;
}

#[tokio::test]
async fn get_unknown_id_returns_none() {
    let registry = HealthRegistry::new();
    assert!(registry.get(&Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn get_by_name_finds_service() {
    let registry = HealthRegistry::new();
    let id = registry
        .register("target", Duration::from_secs(60), None, ok_check())
        .await;

    let health = registry.get_by_name("target").await.unwrap();
    assert_eq!(health.id, id);
    assert_eq!(health.name, "target");
}

#[tokio::test]
async fn get_by_name_returns_none_for_unknown() {
    let registry = HealthRegistry::new();
    assert!(registry.get_by_name("nonexistent").await.is_none());
}

#[tokio::test]
async fn list_returns_all_services() {
    let registry = HealthRegistry::new();
    registry
        .register("a", Duration::from_secs(60), None, ok_check())
        .await;
    registry
        .register("b", Duration::from_secs(60), None, ok_check())
        .await;
    registry
        .register("c", Duration::from_secs(60), None, ok_check())
        .await;

    let services = registry.list().await;
    assert_eq!(services.len(), 3);

    let names: Vec<_> = services.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert!(names.contains(&"c"));
}

#[tokio::test]
async fn list_empty_registry() {
    let registry = HealthRegistry::new();
    assert!(registry.list().await.is_empty());
}

#[tokio::test]
async fn uptime_increases_over_time() {
    let registry = HealthRegistry::new();
    let id = registry
        .register("svc", Duration::from_secs(60), None, ok_check())
        .await;

    let health = registry.get(&id).await.unwrap();
    let uptime1 = health.uptime();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let health = registry.get(&id).await.unwrap();
    let uptime2 = health.uptime();

    assert!(uptime2 > uptime1);
}

#[tokio::test]
async fn background_probe_updates_status() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let should_fail = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&should_fail);

    let check: HealthCheckFn = Box::new(move || {
        let flag = Arc::clone(&flag);
        Box::pin(async move {
            if flag.load(Ordering::Relaxed) {
                Err("went down".to_owned())
            } else {
                Ok(())
            }
        })
    });

    let registry = HealthRegistry::new();
    let id = registry
        .register("flaky", Duration::from_millis(50), None, check)
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::Serving);

    should_fail.store(true, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(150)).await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::NotServing);
    assert_eq!(health.message.as_deref(), Some("went down"));
}

#[tokio::test]
async fn background_probe_recovers_status() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let should_fail = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&should_fail);

    let check: HealthCheckFn = Box::new(move || {
        let flag = Arc::clone(&flag);
        Box::pin(async move {
            if flag.load(Ordering::Relaxed) {
                Err("down".to_owned())
            } else {
                Ok(())
            }
        })
    });

    let registry = HealthRegistry::new();
    let id = registry
        .register("recoverable", Duration::from_millis(50), None, check)
        .await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::NotServing);

    should_fail.store(false, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(150)).await;

    let health = registry.get(&id).await.unwrap();
    assert_eq!(health.status, ServiceStatus::Serving);
    assert!(health.message.is_none());
}

#[tokio::test]
async fn multiple_services_independent_status() {
    let registry = HealthRegistry::new();
    let healthy_id = registry
        .register("healthy", Duration::from_secs(60), None, ok_check())
        .await;
    let unhealthy_id = registry
        .register(
            "unhealthy",
            Duration::from_secs(60),
            None,
            failing_check("error"),
        )
        .await;

    let h = registry.get(&healthy_id).await.unwrap();
    assert_eq!(h.status, ServiceStatus::Serving);

    let u = registry.get(&unhealthy_id).await.unwrap();
    assert_eq!(u.status, ServiceStatus::NotServing);
}

#[test]
fn service_status_equality() {
    assert_eq!(ServiceStatus::Serving, ServiceStatus::Serving);
    assert_eq!(ServiceStatus::NotServing, ServiceStatus::NotServing);
    assert_ne!(ServiceStatus::Serving, ServiceStatus::NotServing);
}

#[test]
fn service_health_clone() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "test".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(30),
        registered_at: Instant::now(),
        version: Some("1.0.0".to_owned()),
        message: None,
    };

    let cloned = health.clone();
    assert_eq!(cloned.id, health.id);
    assert_eq!(cloned.name, health.name);
    assert_eq!(cloned.status, health.status);
    assert_eq!(cloned.version, health.version);
}
