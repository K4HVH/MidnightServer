use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::health::{ServiceHealth, ServiceStatus};
use crate::core::state::AppState;
use crate::proto::health_service_server::HealthService;
use crate::proto::service_health::ServingStatus;
use crate::proto::{OptionalIdRequest, ServiceHealthList};
use tonic::{Request, Response, Status};

pub struct HealthServiceImpl {
    state: Arc<AppState>,
}

impl HealthServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

fn to_proto(h: &ServiceHealth) -> crate::proto::ServiceHealth {
    let status = match h.status {
        ServiceStatus::Serving => ServingStatus::Serving,
        ServiceStatus::NotServing => ServingStatus::NotServing,
    };

    let uptime = h.uptime();
    let interval = h.interval;

    crate::proto::ServiceHealth {
        id: h.id.to_string(),
        name: h.name.clone(),
        status: status.into(),
        interval: Some(prost_types::Duration {
            seconds: interval.as_secs() as i64,
            nanos: interval.subsec_nanos() as i32,
        }),
        uptime: Some(prost_types::Duration {
            seconds: uptime.as_secs() as i64,
            nanos: uptime.subsec_nanos() as i32,
        }),
        version: h.version.clone(),
        message: h.message.clone(),
    }
}

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn list_health_services(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ServiceHealthList>, Status> {
        let services = self.state.health().list().await;
        let services = services.iter().map(to_proto).collect();
        Ok(Response::new(ServiceHealthList { services }))
    }

    async fn get_health_service(
        &self,
        request: Request<OptionalIdRequest>,
    ) -> Result<Response<crate::proto::ServiceHealth>, Status> {
        let id = request.get_ref().id.as_deref().unwrap_or("");

        let health = if id.is_empty() {
            self.state
                .health()
                .get_by_name("server")
                .await
                .ok_or_else(|| AppError::NotFound("server service not registered".into()))?
        } else {
            let uuid = uuid::Uuid::parse_str(id)
                .map_err(|_| AppError::InvalidArgument(format!("invalid uuid: {id}")))?;
            self.state
                .health()
                .get(&uuid)
                .await
                .ok_or_else(|| AppError::NotFound(format!("unknown service: {id}")))?
        };

        Ok(Response::new(to_proto(&health)))
    }
}

#[cfg(test)]
#[path = "../../tests/grpc/health.rs"]
mod tests;
