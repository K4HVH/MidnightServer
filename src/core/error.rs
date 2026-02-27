use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("unauthenticated: {0}")]
    Unauthenticated(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("database: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match &err {
            AppError::NotFound(_) => {
                tracing::warn!(%err);
                Status::not_found(err.to_string())
            }
            AppError::InvalidArgument(_) => {
                tracing::warn!(%err);
                Status::invalid_argument(err.to_string())
            }
            AppError::Unauthenticated(_) => {
                tracing::warn!(%err);
                Status::unauthenticated(err.to_string())
            }
            AppError::PermissionDenied(_) => {
                tracing::warn!(%err);
                Status::permission_denied(err.to_string())
            }
            AppError::AlreadyExists(_) => {
                tracing::warn!(%err);
                Status::already_exists(err.to_string())
            }
            AppError::Internal(_) | AppError::Sqlx(_) | AppError::Anyhow(_) => {
                tracing::error!(%err);
                Status::internal(err.to_string())
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/core/error.rs"]
mod tests;

#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
