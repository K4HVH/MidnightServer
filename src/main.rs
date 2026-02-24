use std::sync::Arc;

use anyhow::Result;
use std::net::SocketAddr;

use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod grpc;
mod proto;
mod state;

use proto::health_service_server::HealthServiceServer;
use state::AppState;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/proto/generated/descriptors.bin"
));

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let config = config::Config::from_env();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .with(fmt::layer())
        .init();

    let addr: SocketAddr = config.listen_addr.parse()?;

    let cors_layer = build_cors_layer(&config);

    let state = AppState::new(config);

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
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .add_service(HealthServiceServer::new(health_service))
        .add_service(reflection_v1)
        .add_service(reflection_v1alpha)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

fn build_cors_layer(config: &config::Config) -> CorsLayer {
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
