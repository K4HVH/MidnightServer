use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::state::AppState;
use crate::proto::{CheckRequest, CheckResponse, check_response::ServingStatus};
use crate::proto::health_service_server::HealthService;
use tonic::{Request, Response, Status};

const KNOWN_SERVICES: &[&str] = &["", "midnightui.HealthService"];

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
        let service = &request.get_ref().service;

        if !service.is_empty() && !KNOWN_SERVICES.contains(&service.as_str()) {
            return Err(AppError::NotFound(format!("unknown service: {service}")).into());
        }

        tracing::debug!(service = %service, "health check");

        let response = CheckResponse {
            status: ServingStatus::Serving.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.state.uptime_secs() as i64,
        };

        Ok(Response::new(response))
    }
}
