/**
 * Common test helpers and utilities for JWT testing.
 * Contains shared functions for creating test data, tokens, and JWKS responses.
 */
use identity_proxy::models::{AudClaim, Claims, RealmAccess, ResourceAccess};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;
use std::collections::HashMap;

// Pre-generated RSA 2048 key pair for testing
pub const TEST_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
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
pub const TEST_KEY_N: &str = "3hhSOJQUyCQSnE2__VTAmDO4Oj4Eejq8VLukSbEXvHfAeslxlFVLIPocYiQlsw87Lqk1aA-Hsup1B8CPjBxoyvJxGDg5FYQJRehDioq50ncxMRm2kl08uAdIsLOi6k5Vf31SwXi2eka_ljZ8oGFz6dfCmKAZLeAWO4gXuAYFaQ7SBWC4AOJ13p-pAWUf7U4rpHHo_PV37-l1n_Rye_gk2XzPZumhj9P_kLk5bU87_mbd-_T5ZekmXh1q8dReqVvXZuu1068uRNBM40xw98HV1HxbBXjUFfDvBEwju38jbAtb6gzgls6MQ2EgxTm_LDIzKKsflRHb_StifCzWAxo7iw";
pub const TEST_KEY_E: &str = "AQAB";

/// Helper to create test claims with configurable issuer, audience, and expiration
pub fn create_test_claims(issuer: &str, audience: &str, exp_offset: i64) -> Claims {
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
pub fn create_token(claims: &Claims, kid: &str) -> String {
    let encoding_key =
        EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes()).expect("Failed to load test key");

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());

    encode(&header, claims, &encoding_key).expect("Failed to encode token")
}

/// Creates a JWKS JSON response with test keys
pub fn create_jwks_json(kid: &str) -> serde_json::Value {
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

/// Creates a Claims instance with custom roles for testing role extraction
pub fn create_claims_with_roles(
    realm_roles: Option<Vec<String>>,
    resource_roles: Option<HashMap<String, Vec<String>>>,
) -> Claims {
    let mut claims = create_test_claims("https://test-issuer.com", "test-audience", 3600);

    claims.realm_access = realm_roles.map(|roles| RealmAccess { roles });
    claims.resource_access = resource_roles.map(|res_map| {
        res_map
            .into_iter()
            .map(|(k, roles)| (k, ResourceAccess { roles }))
            .collect()
    });

    claims
}
