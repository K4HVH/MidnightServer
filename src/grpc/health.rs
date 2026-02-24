use std::sync::Arc;

use crate::proto::{CheckRequest, CheckResponse, check_response::ServingStatus};
use crate::proto::health_service_server::HealthService;
use crate::state::AppState;
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
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let response = CheckResponse {
            status: ServingStatus::Serving.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.state.uptime_secs() as i64,
        };

        Ok(Response::new(response))
    }
}
