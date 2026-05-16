use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Controller error: {0}")]
    Controller(#[from] controller::ControllerError),

    #[error("Monitoring error: {0}")]
    Monitoring(#[from] monitoring::MonitoringError),

    #[error("ML engine error: {0}")]
    MlEngine(#[from] ml_engine::MlError),

    #[error("Optimizer error: {0}")]
    Optimizer(#[from] optimizer::OptimizerError),

    #[error("Resilience error: {0}")]
    Resilience(#[from] resilience::ResilienceError),

    #[error("Analytics error: {0}")]
    Analytics(#[from] analytics::AnalyticsError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Controller(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Monitoring(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::MlEngine(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Optimizer(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Resilience(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Analytics(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
