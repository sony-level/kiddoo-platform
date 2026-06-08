/**
 * Role extraction tests.
 * Tests extraction of roles from realm_access and resource_access claims.
 */
use super::test_helpers::*;
use identity_proxy::middleware::jwt::JwksVerifier;
use identity_proxy::models::{RealmAccess, ResourceAccess};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_extract_roles_from_realm_access() {
    // Create verifier with dummy URLs since we're only testing extract_roles
    let verifier = JwksVerifier::new(
        "http://localhost/jwks".to_string(),
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

#[test]
fn test_extract_roles_from_resource_access() {
    // Create verifier with dummy URLs since we're only testing extract_roles
    let verifier = JwksVerifier::new(
        "http://localhost/jwks".to_string(),
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

#[test]
fn test_extract_roles_deduplication() {
    // Create verifier with dummy URLs since we're only testing extract_roles
    let verifier = JwksVerifier::new(
        "http://localhost/jwks".to_string(),
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
    assert_eq!(
        roles,
        vec![
            "admin".to_string(),
            "developer".to_string(),
            "user".to_string()
        ]
    );
}
