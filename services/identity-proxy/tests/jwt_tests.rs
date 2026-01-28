/**
 * Unit tests for JwksVerifier component.
 * These tests verify JWT verification, key caching, and JWKS handling
 * using mocked JWKS endpoints.
 */
use base64::Engine;
use identity_proxy::middleware::jwt::JwksVerifier;
use identity_proxy::models::{AudClaim, Claims, RealmAccess, ResourceAccess};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Pre-generated RSA 2048 key pair for testing
const TEST_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDeGFI4lBTIJBKc
Tb/9VMCYM7g6PgR6OrxUu6RJsRe8d8B6yXGUVUsg+hxiJCWzDzsuqTVoD4ey6nUH
wI+MHGjK8nEYODkVhAlF6EOKirnSdzExGbaSXTy4B0iws6LqTlV/fVLBeLZ6Rr+W
NnygYXPp18KYoBkt4BY7iBe4BgVpDtIFYLgA4nXen6kBZR/tTiukcej89Xfv6XWf
9HJ7+CTZfM9m6aGP0/+QuTltTzv+Zt379Pll6SZeHWrx1F6pW9dm67XTry5E0Ezj
THD3wdXUfFsFeNQV8O8ETCO7fyNsC1vqDOCWzoxDYSDFOb8sMjMoqx+VEdv9K2J8
LNYDGjuLAgMBAAECggEAHAJDNEFwkYEDG9UuVjt/AnLbU/UISYXoxGLZqDV+QdV0
X8fR3BwZmnIQbEwUuQ09sHfEdXgn6+dnlO+y6r+Wc6m5m41TnaQGK1vMUMeIAcPo
X0HX5YN+qdK5VxeAfz4byDU834tir+8GMGJ0uyEvJhmAwBDIvCFbjGxwWVPhEMuC
iCajZxM4U0nJRifr+IR9Pvui0oSJH2pBOJfwCciahaAfrlaJ0U07iHKO3UldL19L
z+4j9jTUOJgyP6tu6vgqCut81llzyzTENh3M+mh02UvanGW9XMaIAhsrfHIIhCoX
zUYlAm+OrTlbITqhc2OoezkXfQBn0afgtPezW9mMMQKBgQDyr2HgZijyihtV85/8
3K/RLFaLplDFyYfdkI/qnVbp3D3+HCiNAkQqVmoo5SVd/1G3cDtRtXY+yYkUiRSU
+kMB6BVTczKYzMCniSKulpHVt0DINrhrYCU056e4AuYM9RPf9psWuNaBEn3zn5N8
nKyQGhJIIZYmdA3H8E883uq9gwKBgQDqR73/Gugd0AmmaBWKoNmlGhLSYmcBwS1x
75PW5JaBA6sADGWUC/8A8SU9cvB1MEkOqGiZdiGdmfS7wp1TmUH/fls9Pn/M/CXX
YPoeTdC3bQfpZzHX3B0qECd7/o1ZjFnT3+66lDPJ/1vm6TIWJLB6XGLNCnwWHFA8
JHY6WAnzWQKBgGRK7pwaHBn/0UQ4JooNeetr82hLF15l5uw97fv7ggurpUL6yBde
NGV6yOyVpleuSEsS6rDd2TwhdbEy5Xqb4k1LaGTQWryjAYs7NUYJm8NCtFcJpjVZ
yoaOpdV5/TClp80K0RUW1i8JQVwJOp5o8TesTpnYp4DEvV0/hr+VMFWZAoGBAMTH
47+N0x+PaTubu1RIjOcPgnWx74FayBgmOpBKSlwtP4l34C488UDSTAxUKcLU/thP
/iPARLYC5bx71/erB/NZJ7vGbkQ4GnTQ4OVpSQF4lCeo5QXBvcFh9jhA9Gsd5yl+
sx+GcgWd9ox6nPZadN0iEl7VCqrrtzz9B000O49pAoGACgf/A64EWA65ZvrNAV3X
MWwuRUF8gGbgWN9aUBA8Psu8vuL5M71Yd2BgfHHCF+bwdvl4+pkB8SxGqgRFbkZb
mdbkVvomv6lTxtWCsfYktdRRjvs2HsPE/r3on5ydKeW1VOQ/C8aWGbax0KMzykdh
rEgHIVnS7pTQTNC30DR0UMA=
-----END PRIVATE KEY-----"#;

// Corresponding public key components (n, e) in base64url encoding
const TEST_KEY_N: &str = "3hhSOJQUyCQSnE2__VTAmDO4Oj4Eejq8VLukSbEXvHfAeslxlFVLIPocYiQlsw87Lqk1aA-Hsup1B8CPjBxoyvJxGDg5FYQJRehDioq50ncxMRm2kl08uAdIsLOi6k5Vf31SwXi2eka_ljZ8oGFz6dfCmKAZLeAWO4gXuAYFaQ7SBWC4AOJ13p-pAWUf7U4rpHHo_PV37-l1n_Rye_gk2XzPZumhj9P_kLk5bU87_mbd-_T5ZekmXh1q8dReqVvXZuu1068uRNBM40xw98HV1HxbBXjUFfDvBEwju38jbAtb6gzgls6MQ2EgxTm_LDIzKKsflRHb_StifCzWAxo7iw";
const TEST_KEY_E: &str = "AQAB";

/// Helper to create test claims
fn create_test_claims(issuer: &str, audience: &str, exp_offset: i64) -> Claims {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    Claims {
        sub: "test-user-123".to_string(),
        iss: Some(issuer.to_string()),
        aud: Some(AudClaim::Single(audience.to_string())),
        exp: (now as i64 + exp_offset) as usize,
        iat: now,
        email: Some("test@example.com".to_string()),
        preferred_username: Some("testuser".to_string()),
        realm_access: Some(RealmAccess {
            roles: vec!["user".to_string(), "admin".to_string()],
        }),
        resource_access: None,
    }
}

/// Creates a JWT token with the given claims and kid
fn create_token(claims: &Claims, kid: &str) -> String {
    let encoding_key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes())
        .expect("Failed to load test key");
    
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    
    encode(&header, claims, &encoding_key).expect("Failed to encode token")
}

/// Creates a JWKS JSON response with test keys
fn create_jwks_json(kid: &str) -> serde_json::Value {
    json!({
        "keys": [{
            "kty": "RSA",
            "kid": kid,
            "use": "sig",
            "alg": "RS256",
            "n": TEST_KEY_N,
            "e": TEST_KEY_E
        }]
    })
}

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
    let encoding_key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes())
        .expect("Failed to load test key");
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
    assert!(result2.is_ok(), "Second verification should succeed with cache");

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
    assert!(result2.is_ok(), "New key verification should succeed after rotation");
}

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
    assert!(result.is_err(), "Malformed JWKS should cause verification to fail");

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
    assert!(result.is_err(), "Network failure should cause verification to fail");

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
    assert!(result.is_err(), "Server error should cause verification to fail");

    let error = result.unwrap_err();
    assert!(
        matches!(error, identity_proxy::errors::AuthError::InternalError),
        "Expected InternalError, got {:?}",
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
    let modified_header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(header_json.as_bytes());
    
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

#[tokio::test]
async fn test_extract_roles_from_realm_access() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    let mut claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    claims.realm_access = Some(RealmAccess {
        roles: vec!["admin".to_string(), "user".to_string()],
    });

    let roles = verifier.extract_roles(&claims);
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"admin".to_string()));
    assert!(roles.contains(&"user".to_string()));
}

#[tokio::test]
async fn test_extract_roles_from_resource_access() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    let mut claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    let mut resource_access = HashMap::new();
    resource_access.insert(
        "test-client".to_string(),
        ResourceAccess {
            roles: vec!["client-role".to_string()],
        },
    );
    claims.resource_access = Some(resource_access);
    claims.realm_access = None;

    let roles = verifier.extract_roles(&claims);
    assert_eq!(roles.len(), 1);
    assert!(roles.contains(&"client-role".to_string()));
}

#[tokio::test]
async fn test_extract_roles_deduplication() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_jwks_json("test-key-1")))
        .mount(&mock_server)
        .await;

    let verifier = JwksVerifier::new(
        format!("{}/jwks", mock_server.uri()),
        "https://test-issuer.com".to_string(),
        "test-audience".to_string(),
        "test-client".to_string(),
        Duration::from_secs(300),
    );

    let mut claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);
    
    // Add same role in both realm and resource access
    claims.realm_access = Some(RealmAccess {
        roles: vec!["admin".to_string(), "user".to_string()],
    });
    
    let mut resource_access = HashMap::new();
    resource_access.insert(
        "test-client".to_string(),
        ResourceAccess {
            roles: vec!["admin".to_string(), "developer".to_string()],
        },
    );
    claims.resource_access = Some(resource_access);

    let roles = verifier.extract_roles(&claims);
    
    // Should have 3 unique roles (admin should not be duplicated)
    assert_eq!(roles.len(), 3);
    assert!(roles.contains(&"admin".to_string()));
    assert!(roles.contains(&"user".to_string()));
    assert!(roles.contains(&"developer".to_string()));
    
    // Verify roles are sorted
    assert_eq!(roles, vec!["admin".to_string(), "developer".to_string(), "user".to_string()]);
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
    assert!(result.is_ok(), "Should verify with RSA key despite EC key presence");
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
    assert!(result.is_ok(), "Should verify with complete RSA key despite incomplete keys");
}
