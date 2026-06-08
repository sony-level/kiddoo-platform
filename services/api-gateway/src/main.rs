/**
 * API Gateway Service.
 * This is the main entry point for the Kidoo API.
 * It routes requests to downstream microservices and provides
 * centralized authentication, rate limiting, and monitoring.
 */
#[macro_use]
extern crate rocket;

mod config;
mod errors;
mod models;
mod openapi;
mod proxy;
mod routes;

use config::Config;
use openapi::ApiDoc;
use proxy::ProxyClient;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[launch]
fn rocket() -> _ {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let proxy_client = ProxyClient::new(config);

    rocket::build()
        .manage(proxy_client)
        .attach(metrics::PrometheusMetrics::new("api-gateway"))
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
        .mount("/", rocket::routes![metrics::metrics_endpoint])
        .mount(
            "/",
            SwaggerUi::new("/api/v1/swagger-ui/<_..>")
                .url("/api/v1/api-docs/openapi.json", ApiDoc::openapi()),
        )
}
