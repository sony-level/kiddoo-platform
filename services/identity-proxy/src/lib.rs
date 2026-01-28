/**
 * Identity Proxy Library.
 * This library provides authentication services for the Kidoo application.
 *
 * Note: Swagger UI is centralized in the api-gateway service.
 * Access documentation at http://localhost:8000/swagger-ui/
 */
#[macro_use]
extern crate rocket;

pub mod config;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

use config::Config;
use middleware::JwksVerifier;
use services::KeycloakService;
use std::time::Duration;

/// Creates and configures the Rocket instance.
/// This function is public to allow testing.
pub fn create_rocket() -> rocket::Rocket<rocket::Build> {
    let config = Config::from_env().expect("Failed to load configuration");

    let jwks_verifier = JwksVerifier::new(
        config.kc_jwks_url.clone(),
        config.kc_issuer.clone(),
        config.kc_audience.clone(),
        config.kc_client_id.clone(),
        Duration::from_secs(config.jwks_cache_ttl_seconds),
    );

    let keycloak_service = KeycloakService::new(config);

    rocket::build()
        .manage(keycloak_service)
        .manage(jwks_verifier)
        .mount(
            "/api/v1",
            routes![
                routes::health_check,
                routes::authorize,
                routes::callback,
                routes::login,
                routes::refresh,
                routes::logout,
                routes::me,
            ],
        )
}
