/**
 * Identity Proxy Service.
 * This service acts as an authentication proxy between the mobile app and Keycloak.
 * It handles user authentication, token management, and session control.
 */
use identity_proxy::create_rocket;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[rocket::launch]
fn rocket() -> _ {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    create_rocket()
}
