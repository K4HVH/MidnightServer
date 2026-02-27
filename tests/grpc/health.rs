use super::*;
use crate::core::health::{ServiceHealth, ServiceStatus};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[test]
fn to_proto_serving_status() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "test".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(30),
        registered_at: Instant::now(),
        version: Some("1.0.0".to_owned()),
        message: None,
    };

    let proto = to_proto(&health);
    assert_eq!(proto.status(), ServingStatus::Serving);
}

#[test]
fn to_proto_not_serving_status() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "test".to_owned(),
        status: ServiceStatus::NotServing,
        interval: Duration::from_secs(30),
        registered_at: Instant::now(),
        version: None,
        message: Some("down".to_owned()),
    };

    let proto = to_proto(&health);
    assert_eq!(proto.status(), ServingStatus::NotServing);
    assert_eq!(proto.message.as_deref(), Some("down"));
}

#[test]
fn to_proto_preserves_id_as_string() {
    let id = Uuid::new_v4();
    let health = ServiceHealth {
        id,
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    assert_eq!(proto.id, id.to_string());
}

#[test]
fn to_proto_preserves_name() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "my-service".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    assert_eq!(proto.name, "my-service");
}

#[test]
fn to_proto_interval_conversion() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::new(45, 500_000_000),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    let interval = proto.interval.unwrap();
    assert_eq!(interval.seconds, 45);
    assert_eq!(interval.nanos, 500_000_000);
}

#[test]
fn to_proto_uptime_is_positive() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    let uptime = proto.uptime.unwrap();
    assert!(uptime.seconds >= 0);
    assert!(uptime.nanos >= 0);
}

#[test]
fn to_proto_version_present() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: Some("2.5.1".to_owned()),
        message: None,
    };

    let proto = to_proto(&health);
    assert_eq!(proto.version.as_deref(), Some("2.5.1"));
}

#[test]
fn to_proto_version_absent() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    assert!(proto.version.is_none());
}

#[test]
fn to_proto_message_absent_when_healthy() {
    let health = ServiceHealth {
        id: Uuid::new_v4(),
        name: "svc".to_owned(),
        status: ServiceStatus::Serving,
        interval: Duration::from_secs(60),
        registered_at: Instant::now(),
        version: None,
        message: None,
    };

    let proto = to_proto(&health);
    assert!(proto.message.is_none());
}

fn test_config() -> crate::core::config::Config {
    crate::core::config::Config {
        listen_addr: "127.0.0.1:50051".to_owned(),
        log_level: "info".to_owned(),
        log_style: "plain".to_owned(),
        cors_origins: vec!["*".to_owned()],
        database_url: "postgres://localhost/test".to_owned(),
        db_max_connections: 5,
        request_timeout_secs: 30,
    }
}

fn test_pool() -> sqlx::PgPool {
    sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap()
}

#[tokio::test]
async fn list_health_services_returns_all() {
    let state = AppState::new(test_config(), test_pool());
    state
        .health()
        .register(
            "svc1",
            Duration::from_secs(60),
            None,
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;
    state
        .health()
        .register(
            "svc2",
            Duration::from_secs(60),
            None,
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;

    let handler = HealthServiceImpl::new(Arc::clone(&state));
    let response = handler
        .list_health_services(Request::new(()))
        .await
        .unwrap();

    let services = &response.get_ref().services;
    assert_eq!(services.len(), 2);

    let names: Vec<_> = services.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"svc1"));
    assert!(names.contains(&"svc2"));
}

#[tokio::test]
async fn list_health_services_empty() {
    let state = AppState::new(test_config(), test_pool());
    let handler = HealthServiceImpl::new(Arc::clone(&state));

    let response = handler
        .list_health_services(Request::new(()))
        .await
        .unwrap();
    assert!(response.get_ref().services.is_empty());
}

#[tokio::test]
async fn get_health_service_by_uuid() {
    let state = AppState::new(test_config(), test_pool());
    let id = state
        .health()
        .register(
            "myservice",
            Duration::from_secs(60),
            Some("3.0.0".to_owned()),
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;

    let handler = HealthServiceImpl::new(Arc::clone(&state));
    let response = handler
        .get_health_service(Request::new(OptionalIdRequest {
            id: Some(id.to_string()),
        }))
        .await
        .unwrap();

    let svc = response.get_ref();
    assert_eq!(svc.name, "myservice");
    assert_eq!(svc.version.as_deref(), Some("3.0.0"));
}

#[tokio::test]
async fn get_health_service_empty_id_returns_server() {
    let state = AppState::new(test_config(), test_pool());
    state
        .health()
        .register(
            "server",
            Duration::from_secs(60),
            Some("0.1.0".to_owned()),
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;

    let handler = HealthServiceImpl::new(Arc::clone(&state));
    let response = handler
        .get_health_service(Request::new(OptionalIdRequest { id: None }))
        .await
        .unwrap();

    assert_eq!(response.get_ref().name, "server");
}

#[tokio::test]
async fn get_health_service_no_id_field_returns_server() {
    let state = AppState::new(test_config(), test_pool());
    state
        .health()
        .register(
            "server",
            Duration::from_secs(60),
            None,
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;

    let handler = HealthServiceImpl::new(Arc::clone(&state));
    let response = handler
        .get_health_service(Request::new(OptionalIdRequest {
            id: Some(String::new()),
        }))
        .await
        .unwrap();

    assert_eq!(response.get_ref().name, "server");
}

#[tokio::test]
async fn get_health_service_invalid_uuid_returns_error() {
    let state = AppState::new(test_config(), test_pool());
    let handler = HealthServiceImpl::new(Arc::clone(&state));

    let result = handler
        .get_health_service(Request::new(OptionalIdRequest {
            id: Some("not-a-uuid".to_owned()),
        }))
        .await;

    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn get_health_service_unknown_uuid_returns_not_found() {
    let state = AppState::new(test_config(), test_pool());
    let handler = HealthServiceImpl::new(Arc::clone(&state));

    let result = handler
        .get_health_service(Request::new(OptionalIdRequest {
            id: Some(Uuid::new_v4().to_string()),
        }))
        .await;

    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn get_health_service_no_server_registered_returns_not_found() {
    let state = AppState::new(test_config(), test_pool());
    let handler = HealthServiceImpl::new(Arc::clone(&state));

    let result = handler
        .get_health_service(Request::new(OptionalIdRequest { id: None }))
        .await;

    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}
