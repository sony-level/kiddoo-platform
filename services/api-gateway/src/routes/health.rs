/**
 * Health check routes for API Gateway.
 * This module provides health monitoring endpoints for the gateway
 * and its downstream services.
 */
use crate::models::{HealthResponse, ServiceStatus};
use crate::proxy::ProxyClient;
use rocket::State;
use rocket::get;
use rocket::serde::json::Json;

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Gateway health status", body = HealthResponse)
    )
)]
#[get("/health")]
pub async fn health_check(proxy: &State<ProxyClient>) -> Json<HealthResponse> {
    let identity_healthy = proxy.check_identity_proxy_health().await;

    Json(HealthResponse {
        status: "ok".to_string(),
        service: "api-gateway".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services: vec![ServiceStatus {
            name: "identity-proxy".to_string(),
            status: if identity_healthy {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            url: std::env::var("IDENTITY_PROXY_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
        }],
    })
}
