use super::*;
use tonic::Code;

#[test]
fn not_found_maps_to_grpc_not_found() {
    let status: Status = AppError::NotFound("missing".into()).into();
    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("missing"));
}

#[test]
fn invalid_argument_maps_to_grpc_invalid_argument() {
    let status: Status = AppError::InvalidArgument("bad input".into()).into();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("bad input"));
}

#[test]
fn internal_maps_to_grpc_internal() {
    let status: Status = AppError::Internal("boom".into()).into();
    assert_eq!(status.code(), Code::Internal);
    assert!(status.message().contains("boom"));
}

#[test]
fn unauthenticated_maps_to_grpc_unauthenticated() {
    let status: Status = AppError::Unauthenticated("no token".into()).into();
    assert_eq!(status.code(), Code::Unauthenticated);
    assert!(status.message().contains("no token"));
}

#[test]
fn permission_denied_maps_to_grpc_permission_denied() {
    let status: Status = AppError::PermissionDenied("forbidden".into()).into();
    assert_eq!(status.code(), Code::PermissionDenied);
    assert!(status.message().contains("forbidden"));
}

#[test]
fn already_exists_maps_to_grpc_already_exists() {
    let status: Status = AppError::AlreadyExists("duplicate".into()).into();
    assert_eq!(status.code(), Code::AlreadyExists);
    assert!(status.message().contains("duplicate"));
}

#[test]
fn anyhow_error_maps_to_grpc_internal() {
    let err = AppError::Anyhow(anyhow::anyhow!("something broke"));
    let status: Status = err.into();
    assert_eq!(status.code(), Code::Internal);
}

#[test]
fn display_formats_correctly() {
    assert_eq!(
        AppError::NotFound("user 42".into()).to_string(),
        "not found: user 42"
    );
    assert_eq!(
        AppError::InvalidArgument("bad id".into()).to_string(),
        "invalid argument: bad id"
    );
    assert_eq!(
        AppError::Internal("oops".into()).to_string(),
        "internal: oops"
    );
}

#[test]
fn from_anyhow_conversion() {
    let anyhow_err = anyhow::anyhow!("test error");
    let app_err: AppError = anyhow_err.into();
    assert!(matches!(app_err, AppError::Anyhow(_)));
}
