use rocket::Request;
/**
 * Error handling module for API Gateway.
 * This module defines error types for gateway operations and proxy failures.
 */
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[allow(dead_code)]
    #[error("Request timeout")]
    Timeout,

    #[error("Bad gateway: {0}")]
    BadGateway(String),

    #[error("Internal server error")]
    InternalError,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code identifier
    #[schema(example = "service_unavailable")]
    pub error: String,
    /// Human-readable error message
    #[schema(example = "Service unavailable")]
    pub message: String,
}

impl<'r> Responder<'r, 'static> for GatewayError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let (status, error_type) = match &self {
            GatewayError::ServiceUnavailable(_) => {
                (Status::ServiceUnavailable, "service_unavailable")
            }
            GatewayError::Timeout => (Status::GatewayTimeout, "timeout"),
            GatewayError::BadGateway(_) => (Status::BadGateway, "bad_gateway"),
            GatewayError::InternalError => (Status::InternalServerError, "internal_error"),
        };

        let response = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        };

        (status, Json(response)).respond_to(req)
    }
}
