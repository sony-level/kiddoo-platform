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
use services::KeycloakService;

/// Creates and configures the Rocket instance.
/// This function is public to allow testing.
pub fn create_rocket() -> rocket::Rocket<rocket::Build> {
    let config = Config::from_env().expect("Failed to load configuration");
    let keycloak_service = KeycloakService::new(config);

    rocket::build().manage(keycloak_service).mount(
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
