use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use std::net::SocketAddr;

use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod core;
mod grpc;
mod proto;

use core::state::AppState;
use proto::health_service_server::HealthServiceServer;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/proto/generated/descriptors.bin"
));

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let config = core::config::Config::from_env();
    core::logging::init(&config);

    let addr: SocketAddr = config.listen_addr.parse()?;

    let cors_layer = build_cors_layer(&config);

    let db = core::db::create_pool(&config.database_url, config.db_max_connections).await?;
    core::db::run_migrations(&db).await?;

    let state = AppState::new(config, db);

    state
        .health()
        .register(
            "server",
            Duration::from_secs(60),
            Some(env!("CARGO_PKG_VERSION").to_owned()),
            Box::new(|| Box::pin(async { Ok(()) })),
        )
        .await;

    let db_version = get_db_version(state.db()).await;
    let db_pool = state.db().clone();
    state
        .health()
        .register(
            "database",
            Duration::from_secs(30),
            db_version,
            Box::new(move || {
                let pool = db_pool.clone();
                Box::pin(async move {
                    sqlx::query!("SELECT 1 as health_check")
                        .fetch_one(&pool)
                        .await
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                })
            }),
        )
        .await;

    let health_service = grpc::health::HealthServiceImpl::new(Arc::clone(&state));

    let reflection_v1 = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let reflection_v1alpha = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    tracing::info!("MidnightServer listening on {addr}");

    Server::builder()
        .accept_http1(true)
        .timeout(Duration::from_secs(state.config().request_timeout_secs))
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .layer(
            TraceLayer::new_for_grpc()
                .make_span_with(|req: &http::Request<_>| {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::info_span!("grpc", %request_id, method = %req.uri().path())
                })
                .on_request(|_req: &http::Request<_>, _span: &tracing::Span| {
                    tracing::info!("request received");
                })
                .on_response(
                    |res: &http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        let status = res
                            .headers()
                            .get("grpc-status")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("0");
                        tracing::info!(latency_ms = latency.as_millis(), grpc_status = %status, "response sent");
                    },
                )
                .on_failure(
                    |err: tower_http::classify::GrpcFailureClass,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::error!(?err, latency_ms = latency.as_millis(), "request failed");
                    },
                ),
        )
        .add_service(HealthServiceServer::new(health_service))
        .add_service(reflection_v1)
        .add_service(reflection_v1alpha)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

fn build_cors_layer(config: &core::config::Config) -> CorsLayer {
    let origin = if config.cors_is_permissive() {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new().allow_origin(AllowOrigin::list(origins))
    };

    origin
        .allow_headers(Any)
        .allow_methods(Any)
        .expose_headers(Any)
}

async fn get_db_version(pool: &sqlx::PgPool) -> Option<String> {
    sqlx::query_scalar!("SELECT version()")
        .fetch_one(pool)
        .await
        .ok()
        .flatten()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
