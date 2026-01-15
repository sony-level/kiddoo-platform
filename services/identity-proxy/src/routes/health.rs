/**
 * Health check route module.
 * This module provides service health monitoring endpoints for
 * load balancers and orchestration systems.
 */
use crate::models::HealthResponse;
use rocket::get;
use rocket::serde::json::Json;

/**
 * Returns the current health status of the identity-proxy service.
 * Used by load balancers and monitoring systems to verify service availability.
 *
 * ## Route: GET /api/v1/health
 *
 * ## Security: 🔓 Public (no authentication required)
 *
 * ## Response (200 OK)
 * ```json
 * {
 *   "status": "ok",
 *   "service": "identity-proxy",
 *   "version": "0.1.0"
 * }
 * ```
 */
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
#[get("/health")]
pub fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "identity-proxy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
