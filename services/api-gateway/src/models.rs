/**
 * Models module for API Gateway.
 * This module re-exports models from downstream services and defines gateway-specific models.
 */
use serde::Serialize;
use utoipa::ToSchema;

// Re-export authentication models from identity-proxy
pub use identity_proxy::models::{
    LoginRequest, LoginResponse, LogoutRequest, MessageResponse, RefreshRequest, UserResponse,
};

// ============================================================================
// Gateway Models
// ============================================================================

/**
 * Health check response for the gateway.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    #[schema(example = "ok")]
    pub status: String,
    /// Service name
    #[schema(example = "api-gateway")]
    pub service: String,
    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Downstream services status
    pub services: Vec<ServiceStatus>,
}

/**
 * Status of a downstream service.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceStatus {
    /// Service name
    #[schema(example = "identity-proxy")]
    pub name: String,
    /// Service health status
    #[schema(example = "healthy")]
    pub status: String,
    /// Service URL
    #[schema(example = "http://localhost:8001")]
    pub url: String,
}
