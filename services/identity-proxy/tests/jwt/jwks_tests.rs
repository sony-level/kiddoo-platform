/**
 * JWKS endpoint and response handling tests.
 * Tests various JWKS server responses, errors, and key formats.
 */
use super::test_helpers::*;
use identity_proxy::middleware::jwt::JwksVerifier;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_malformed_jwks_response() {
    // Setup mock JWKS server with invalid JSON
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json {{{"))
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

    // Create valid token
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify token should fail due to malformed JWKS
    let result = verifier.verify(&token).await;
    assert!(
        result.is_err(),
        "Malformed JWKS should cause verification to fail"
    );

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InternalError),
        "Expected InternalError, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_network_failure_during_jwks_fetch() {
    // Create verifier pointing to non-existent server
    let verifier = JwksVerifier::new(
        "http://localhost:9999/jwks".to_string(), // Non-existent server
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    // Create valid token
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify token should fail due to network error
    let result = verifier.verify(&token).await;
    assert!(
        result.is_err(),
        "Network failure should cause verification to fail"
    );

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InternalError),
        "Expected InternalError, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_jwks_server_error_response() {
    // Setup mock JWKS server returning 500 error
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(500))
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

    // Create valid token
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify token should fail due to server error
    let result = verifier.verify(&token).await;
    assert!(
        result.is_err(),
        "Server error should cause verification to fail"
    );

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InternalError),
        "Expected InternalError, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_jwks_with_non_rsa_keys() {
    // Setup mock JWKS server with mixed key types
    let mock_server = MockServer::start().await;

    let mixed_jwks = json!({
        "keys": [
            {
                "kty": "EC",  // Elliptic curve - should be skipped
                "kid": "ec-key",
                "use": "sig",
                "alg": "ES256"
            },
            {
                "kty": "RSA",
                "kid": "test-key-1",
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N,
                "e": TEST_KEY_E
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mixed_jwks))
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

    // Create valid token with RSA key
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Should successfully verify with RSA key, ignoring EC key
    let result = verifier.verify(&token).await;
    assert!(
        result.is_ok(),
        "Should verify with RSA key despite EC key presence"
    );
}

#[tokio::test]
async fn test_jwks_with_missing_components() {
    // Setup mock JWKS server with incomplete RSA keys
    let mock_server = MockServer::start().await;

    let incomplete_jwks = json!({
        "keys": [
            {
                "kty": "RSA",
                "kid": "incomplete-key",
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N
                // Missing "e" component - should be skipped
            },
            {
                "kty": "RSA",
                // Missing "kid" - should be skipped
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N,
                "e": TEST_KEY_E
            },
            {
                "kty": "RSA",
                "kid": "test-key-1",
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N,
                "e": TEST_KEY_E
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(incomplete_jwks))
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

    // Create valid token with complete key
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Should successfully verify with complete key
    let result = verifier.verify(&token).await;
    assert!(
        result.is_ok(),
        "Should verify with complete RSA key despite incomplete keys"
    );
}
