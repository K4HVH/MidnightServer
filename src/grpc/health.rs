use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::health::ServiceStatus;
use crate::core::state::AppState;
use crate::proto::{CheckRequest, CheckResponse, check_response::ServingStatus};
use crate::proto::health_service_server::HealthService;
use tonic::{Request, Response, Status};

pub struct HealthServiceImpl {
    state: Arc<AppState>,
}

impl HealthServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let service = request.get_ref().service.as_deref().unwrap_or("");
        let registry = self.state.health();

        let health = registry.check(service).await.ok_or_else(|| {
            AppError::NotFound(format!("unknown service: {service}"))
        })?;

        tracing::debug!(service = %service, ?health.status, "health check");

        let proto_status = match health.status {
            ServiceStatus::Serving => ServingStatus::Serving,
            ServiceStatus::NotServing => ServingStatus::NotServing,
        };

        let response = CheckResponse {
            status: proto_status.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.state.uptime_secs() as i64,
            message: health.message,
        };

        Ok(Response::new(response))
    }
}
