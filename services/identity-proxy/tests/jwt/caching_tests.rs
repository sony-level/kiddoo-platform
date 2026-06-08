/**
 * JWKS caching tests.
 * Tests key caching behavior, cache expiration, and cache hits.
 */
use super::test_helpers::*;
use identity_proxy::middleware::jwt::JwksVerifier;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_cache_expiration() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .expect(2) // Should fetch twice: initial and after cache expiration
        .mount(&mock_server)
        .await;

    // Create verifier with very short TTL
    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_millis(100), // 100ms TTL
    );

    // Create valid token
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token = create_token(&claims, "test-key-1");

    // First verification - should fetch JWKS
    let result1 = verifier.verify(&token).await;
    assert!(result1.is_ok(), "First verification should succeed");

    // Wait for cache to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Second verification - should fetch JWKS again
    let result2 = verifier.verify(&token).await;
    assert!(result2.is_ok(), "Second verification should succeed");

    // Mock expectations are verified automatically on drop
}

#[tokio::test]
async fn test_cache_hit() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .expect(1) // Should only fetch once due to caching
        .mount(&mock_server)
        .await;

    // Create verifier with long TTL
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

    // First verification - should fetch JWKS
    let result1 = verifier.verify(&token).await;
    assert!(result1.is_ok(), "First verification should succeed");

    // Second verification - should use cached JWKS
    let result2 = verifier.verify(&token).await;
    assert!(
        result2.is_ok(),
        "Second verification should succeed with cache"
    );

    // Mock expectations are verified automatically
}

#[tokio::test]
async fn test_key_rotation() {
    // Setup mock JWKS server
    let mock_server = MockServer::start().await;

    // Initially serve JWKS with only the first key
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Create verifier with short TTL for key rotation
    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_millis(100),
    );

    // Verify token with first key
    let claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let token1 = create_token(&claims, "test-key-1");
    let result1 = verifier.verify(&token1).await;
    assert!(result1.is_ok(), "First key verification should succeed");

    // Simulate key rotation - serve JWKS with both keys
    let rotated_jwks = json!({
        "keys": [
            {
                "kty": "RSA",
                "kid": "test-key-1",
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N,
                "e": TEST_KEY_E
            },
            {
                "kty": "RSA",
                "kid": "test-key-2",
                "use": "sig",
                "alg": "RS256",
                "n": TEST_KEY_N,
                "e": TEST_KEY_E
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rotated_jwks))
        .mount(&mock_server)
        .await;

    // Wait for cache to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify token with new key
    let token2 = create_token(&claims, "test-key-2");
    let result2 = verifier.verify(&token2).await;
    assert!(
        result2.is_ok(),
        "New key verification should succeed after rotation"
    );
}
