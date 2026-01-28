# JWT Tests Organization

This directory contains organized JWT verification tests for the identity-proxy service.

## Structure

```
tests/
├── jwt_tests_main.rs       # Main test entry point
└── jwt/
    ├── mod.rs              # Module organization
    ├── test_helpers.rs     # Common test utilities and helpers
    ├── verification_tests.rs    # Basic JWT verification tests
    ├── expiration_tests.rs      # Token expiration handling tests
    ├── security_tests.rs        # Signature and algorithm security tests
    ├── caching_tests.rs         # JWKS key caching tests
    ├── jwks_tests.rs            # JWKS endpoint and response tests
    └── role_extraction_tests.rs # Role extraction from claims tests
```

## Test Categories

### 1. **test_helpers.rs**
Common utilities and test data creation functions:
- RSA key pair constants
- `create_test_claims()` - Create test JWT claims
- `create_token()` - Generate signed JWT tokens
- `create_jwks_json()` - Create JWKS responses
- `create_claims_with_roles()` - Create claims with custom roles

### 2. **verification_tests.rs**
Basic JWT verification scenarios:
- ✅ Successful token verification
- ❌ Invalid issuer rejection
- ❌ Invalid audience rejection

### 3. **expiration_tests.rs**
Token expiration handling:
- ❌ Expired token rejection

### 4. **security_tests.rs**
Security-related validations:
- ❌ Invalid signature detection
- ❌ Missing kid header rejection
- ❌ Unknown kid rejection
- ❌ Invalid algorithm rejection

### 5. **caching_tests.rs**
JWKS key caching behavior:
- ✅ Cache expiration and refresh
- ✅ Cache hit efficiency
- ✅ Key rotation support

### 6. **jwks_tests.rs**
JWKS endpoint and response handling:
- ❌ Malformed JWKS response handling
- ❌ Network failure handling
- ❌ Server error response handling
- ✅ Non-RSA key filtering
- ✅ Missing component handling

### 7. **role_extraction_tests.rs**
Role extraction from JWT claims:
- ✅ Realm access roles
- ✅ Resource access roles
- ✅ Role deduplication

## Running Tests

Run all JWT tests:
```bash
cargo test --test jwt_tests_main
```

Run specific test category:
```bash
cargo test --test jwt_tests_main jwt::verification_tests
cargo test --test jwt_tests_main jwt::security_tests
cargo test --test jwt_tests_main jwt::caching_tests
```

Run a specific test:
```bash
cargo test --test jwt_tests_main test_successful_token_verification
```

## Test Coverage

Total: **19 tests** covering:
- ✅ Token verification (3 tests)
- ✅ Expiration handling (1 test)
- ✅ Security validation (4 tests)
- ✅ Caching behavior (3 tests)
- ✅ JWKS handling (5 tests)
- ✅ Role extraction (3 tests)

## Adding New Tests

1. Identify the appropriate category file
2. Add your test function with `#[tokio::test]` or `#[test]` attribute
3. Use helpers from `test_helpers.rs`
4. Follow existing patterns for consistency

Example:
```rust
#[tokio::test]
async fn test_my_new_feature() {
    let mock_server = MockServer::start().await;
    // ... setup mock
    
    let verifier = JwksVerifier::new(/* ... */);
    let claims = create_test_claims(/* ... */);
    let token = create_token(&claims, "test-key-1");
    
    let result = verifier.verify(&token).await;
    assert!(result.is_ok());
}
```

## Migration Note

This organization replaces the previous monolithic `jwt_tests.rs` file (832 lines) with a modular structure for better maintainability and understanding.
