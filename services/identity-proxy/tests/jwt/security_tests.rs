/**
 * Security-related JWT tests.
 * Tests signature validation, algorithm verification, and other security aspects.
 */
use super::test_helpers::*;
use base64::Engine;
use identity_proxy::middleware::jwt::JwksVerifier;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_invalid_signature() {
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

    // Create token and tamper with signature
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Tamper with the signature
    let parts: Vec<&str> = token.split('.').collect();
    let tampered_token = format!("{}.{}.{}tampered", parts[0], parts[1], parts[2]);

    // Verify token should fail
    let result = verifier.verify(&tampered_token).await;
    assert!(result.is_err(), "Token with invalid signature should fail");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_missing_kid_header() {
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

    // Create token without kid in header
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let encoding_key =
        EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes()).expect("Failed to load test key");
    let header = Header::new(Algorithm::RS256); // No kid set
    let token = encode(&header, &claims, &encoding_key).expect("Failed to encode");

    // Verify token should fail
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "Token without kid should fail");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_unknown_kid() {
    // Setup mock JWKS server with one key
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("known-key")))
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

    // Create token with unknown kid
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "unknown-key");

    // Verify token should fail
    let result = verifier.verify(&token).await;
    assert!(result.is_err(), "Token with unknown kid should fail");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}

#[tokio::test]
async fn test_invalid_algorithm() {
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

    // Create token with RS256 and manually modify header to claim HS256
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // Parse and modify the header to claim HS256
    let parts: Vec<&str> = token.split('.').collect();
    let mut modified_header = Header::new(Algorithm::HS256);
    modified_header.kid = Some("test-key-1".to_string());
    let header_json = serde_json::to_string(&modified_header).unwrap();
    let modified_header_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());

    let modified_token = format!("{}.{}.{}", modified_header_b64, parts[1], parts[2]);

    // Verify should fail due to algorithm mismatch
    let result = verifier.verify(&modified_token).await;
    assert!(result.is_err(), "Non-RS256 algorithm should fail");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InvalidToken),
        "Expected InvalidToken error, got {:?}",
        error
    );
}
