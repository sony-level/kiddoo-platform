/**
 * Integration tests for identity-proxy routes.
 * These tests verify the HTTP endpoints behavior using Rocket's testing utilities.
 */
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment variables (safe, runs once)
fn init_test_env() {
    INIT.call_once(|| {
        // SAFETY: This is only called once at test initialization
        // before any threads are spawned
        unsafe {
            std::env::set_var("KEYCLOAK_URL", "http://localhost:8080");
            std::env::set_var("KEYCLOAK_REALM", "test");
            std::env::set_var("KEYCLOAK_CLIENT_ID", "test-client");
            std::env::set_var("KEYCLOAK_CLIENT_SECRET", "test-secret");
            std::env::set_var("JWT_SECRET", "test-jwt-secret-key-min-32-chars!!");
        }
    });
}

/// Creates a test client for the identity-proxy service
async fn create_test_client() -> Client {
    init_test_env();

    let rocket = identity_proxy::create_rocket();
    Client::tracked(rocket)
        .await
        .expect("Failed to create test client")
}

#[tokio::test]
async fn test_health_check_returns_ok() {
    let client = create_test_client().await;

    let response = client.get("/api/v1/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"service\":\"identity-proxy\""));
}

#[tokio::test]
async fn test_health_check_returns_json() {
    let client = create_test_client().await;

    let response = client.get("/api/v1/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
}

#[tokio::test]
async fn test_login_requires_json_body() {
    let client = create_test_client().await;

    let response = client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body("{}")
        .dispatch()
        .await;

    // Should fail because email/password are missing, but endpoint exists
    assert!(response.status() != Status::NotFound);
}

#[tokio::test]
async fn test_login_with_empty_credentials() {
    let client = create_test_client().await;

    let response = client
        .post("/api/v1/auth/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "", "password": ""}"#)
        .dispatch()
        .await;

    // Keycloak will reject empty credentials
    // We expect either 401 or 502 (if Keycloak is not running)
    let status = response.status();
    assert!(
        status == Status::Unauthorized || status == Status::BadGateway,
        "Expected 401 or 502, got {:?}",
        status
    );
}

#[tokio::test]
async fn test_refresh_requires_json_body() {
    let client = create_test_client().await;

    let response = client
        .post("/api/v1/auth/refresh")
        .header(ContentType::JSON)
        .body(r#"{"refresh_token": "invalid-token"}"#)
        .dispatch()
        .await;

    // Should fail with invalid token, but endpoint exists
    assert!(response.status() != Status::NotFound);
}

#[tokio::test]
async fn test_logout_requires_json_body() {
    let client = create_test_client().await;

    let response = client
        .post("/api/v1/auth/logout")
        .header(ContentType::JSON)
        .body(r#"{"refresh_token": "some-token"}"#)
        .dispatch()
        .await;

    // Should fail with invalid token, but endpoint exists
    assert!(response.status() != Status::NotFound);
}

#[tokio::test]
async fn test_me_requires_authorization_header() {
    let client = create_test_client().await;

    let response = client.post("/api/v1/auth/me").dispatch().await;

    // Should return 401 because no Authorization header
    assert_eq!(response.status(), Status::Unauthorized);
}

#[tokio::test]
async fn test_me_rejects_invalid_token() {
    let client = create_test_client().await;

    let response = client
        .post("/api/v1/auth/me")
        .header(rocket::http::Header::new(
            "Authorization",
            "Bearer invalid-jwt-token",
        ))
        .dispatch()
        .await;

    // Should return 401 for invalid JWT token
    assert_eq!(response.status(), Status::Unauthorized);
}

#[tokio::test]
async fn test_me_rejects_malformed_authorization_header() {
    let client = create_test_client().await;

    // Missing "Bearer " prefix
    let response = client
        .post("/api/v1/auth/me")
        .header(rocket::http::Header::new(
            "Authorization",
            "just-a-token-without-bearer",
        ))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized);
}
