/**
 * Basic JWT verification tests.
 * Tests successful token verification scenarios.
 */
use super::test_helpers::*;
use identity_proxy::middleware::jwt::JwksVerifier;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_successful_token_verification() {
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

    // Create valid token
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify token
    let result = verifier.verify(&token).await;
    assert!(result.is_ok(), "Token verification should succeed");

    let verified_claims = result.unwrap();
    assert_eq!(verified_claims.sub, "test-user-123");
    assert_eq!(verified_claims.email, Some("test@example.com".to_string()));
}

#[tokio::test]
async fn test_invalid_issuer() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    // Create verifier expecting specific issuer
    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://expected-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    // Create token with different issuer
    let claims = create_test_claims("https://wrong-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify should fail due to issuer mismatch
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "Wrong issuer should fail verification");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_invalid_audience() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    // Create verifier expecting specific audience
    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "expected-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    // Create token with different audience
    let claims = create_test_claims("https://test-issuer.com", "wrong-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Verify should fail due to audience mismatch
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "Wrong audience should fail verification");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}
