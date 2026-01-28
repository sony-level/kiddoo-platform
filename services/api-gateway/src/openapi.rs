/**
 * OpenAPI documentation module for API Gateway.
 * This module provides centralized OpenAPI/Swagger documentation
 * for all microservices accessible through the gateway.
 */
use utoipa::OpenApi;

use crate::errors::ErrorResponse;
use crate::models::{
    HealthResponse, LoginRequest, LoginResponse, LogoutRequest, MessageResponse, RefreshRequest,
    ServiceStatus, UserResponse,
};
use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Kidoo API",
        version = "1.0.0",
        description = "API for the Kidoo babysitting application. This documentation covers all available endpoints across microservices.",
        contact(
            name = "Kidoo Team",
            email = "support@kidoo.app"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "/api/v1", description = "API v1")
    ),
    paths(
        routes::health::health_check,
        routes::auth::authorize,
        routes::auth::callback,
        routes::auth::login,
        routes::auth::refresh,
        routes::auth::logout,
        routes::auth::me,
    ),
    components(
        schemas(
            // Gateway schemas
            HealthResponse,
            ServiceStatus,
            ErrorResponse,
            // Authentication schemas
            LoginRequest,
            LoginResponse,
            RefreshRequest,
            LogoutRequest,
            MessageResponse,
            UserResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "Identity Proxy", description = "Authentication endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::builder()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
