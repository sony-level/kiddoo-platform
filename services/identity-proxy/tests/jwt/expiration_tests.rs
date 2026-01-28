/**
 * Token expiration and timing tests.
 * Tests JWT expiration handling.
 */
use super::test_helpers::*;
use identity_proxy::middleware::jwt::JwksVerifier;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_expired_token() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    // Create verifier
    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    // Create expired token (exp in the past)
    let claims = create_test_claims("https://test-issuer.com", "test-audience", -3600);
    let token = create_token(&claims, "test-key-1");

    // Verify token should fail
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "Expired token should fail verification");

    // Check that the error is TokenExpired
    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::TokenExpired),
        "Expected TokenExpired error, got {:?}",
        error
    );
}
