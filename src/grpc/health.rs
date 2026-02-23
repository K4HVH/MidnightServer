use std::time::Instant;

use crate::proto::{CheckRequest, CheckResponse, check_response::ServingStatus};
use crate::proto::health_service_server::HealthService;
use tonic::{Request, Response, Status};

pub struct HealthServiceImpl {
    started_at: Instant,
}

impl HealthServiceImpl {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn check(
        &self,
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let response = CheckResponse {
            status: ServingStatus::Serving.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.started_at.elapsed().as_secs() as i64,
        };

        Ok(Response::new(response))
    }
}
