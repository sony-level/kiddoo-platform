/**
 * JWT middleware module.
 * This module provides request guards for JWT token validation and
 * extraction of authenticated user information from requests.
 */
use crate::errors::AuthError;
use crate::models::Claims;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/**
 * JWT request guard for protected routes.
 * Automatically validates JWT tokens and extracts user information
 * from the Authorization header.
 */
#[derive(Debug)]
pub struct JwtGuard {
    pub kc_sub: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    alg: Option<String>,
    n: Option<String>,
    e: Option<String>,
}

// NOTE: We do not derive `Debug` here because `DecodingKey` does not implement it.
// This is intentional (sensitive key material). If logging is needed, implement
// `Debug` manually and expose only safe metadata (e.g. key count, kid, timestamp).

// #[derive(Debug)]
struct CachedJwks {
    fetched_at: Instant,
    keys_by_kid: HashMap<String, Arc<DecodingKey>>,
}

//#[derive(Debug)]
pub struct JwksVerifier {
    jwks_url: String,
    issuer: String,
    audience: String,
    client_id: String,
    ttl: Duration,
    http: Client,
    cache: RwLock<Option<CachedJwks>>,
}

impl JwksVerifier {
    pub fn new(
        jwks_url: String,
        issuer: String,
        audience: String,
        client_id: String,
        ttl: Duration,
    ) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|err| {
                eprintln!(
                    "Failed to create configured HTTP client ({}); falling back to default client",
                    err
                );
                Client::new()
            });

        Self {
            jwks_url,
            issuer,
            audience,
            client_id,
            ttl,
            http,
            cache: RwLock::new(None),
        }
    }

    fn cache_valid(&self, cached: &CachedJwks) -> bool {
        cached.fetched_at.elapsed() < self.ttl
    }

    async fn refresh_jwks(&self) -> Result<CachedJwks, AuthError> {
        let resp = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|_| AuthError::InternalError)?;

        if !resp.status().is_success() {
            return Err(AuthError::InternalError);
        }

        let jwks = resp
            .json::<Jwks>()
            .await
            .map_err(|_| AuthError::InternalError)?;

        let mut keys_by_kid = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty != "RSA" {
                continue;
            }

            let kid = match jwk.kid {
                Some(k) => k,
                None => continue,
            };

            let n = match jwk.n {
                Some(v) => v,
                None => continue,
            };

            let e = match jwk.e {
                Some(v) => v,
                None => continue,
            };

            let key =
                DecodingKey::from_rsa_components(&n, &e).map_err(|_| AuthError::InternalError)?;
            keys_by_kid.insert(kid, Arc::new(key));
        }

        Ok(CachedJwks {
            fetched_at: Instant::now(),
            keys_by_kid,
        })
    }

    async fn decoding_key_for_kid(&self, kid: &str) -> Result<Arc<DecodingKey>, AuthError> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if self.cache_valid(cached) {
                    if let Some(key) = cached.keys_by_kid.get(kid) {
                        return Ok(Arc::clone(key));
                    }
                }
            }
        }

        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.as_ref() {
            if self.cache_valid(cached) {
                if let Some(key) = cached.keys_by_kid.get(kid) {
                    return Ok(Arc::clone(key));
                }
            }
        }

        let refreshed = self.refresh_jwks().await?;
        let key = refreshed
            .keys_by_kid
            .get(kid)
            .ok_or(AuthError::InvalidToken)?
            .clone();
        *cache = Some(refreshed);
        Ok(key)
    }

    pub async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(AuthError::InvalidToken);
        }

        let kid = header.kid.ok_or(AuthError::InvalidToken)?;
        let key = self.decoding_key_for_kid(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);

        let token_data =
            decode::<Claims>(token, &key, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })?;

        Ok(token_data.claims)
    }

    pub fn extract_roles(&self, claims: &Claims) -> Vec<String> {
        let mut roles = Vec::new();

        if let Some(ra) = claims.realm_access.as_ref() {
            roles.extend(ra.roles.clone());
        }

        if let Some(resources) = claims.resource_access.as_ref() {
            if let Some(access) = resources.get(&self.client_id) {
                roles.extend(access.roles.clone());
            }
        }

        roles.sort();
        roles.dedup();
        roles
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtGuard {
    type Error = AuthError;

    /**
     * Extracts and validates JWT token from the request.
     * Parses the Authorization header, decodes the JWT, and extracts user claims.
     *
     * # Arguments
     * * `request` - The incoming HTTP request
     *
     * # Returns
     * * `Outcome::Success(JwtGuard)` - Valid token with user information
     * * `Outcome::Error` - Authentication failure with appropriate error
     */
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => {
                return Outcome::Error((Status::Unauthorized, AuthError::MissingAuthHeader));
            }
        };

        let verifier = match request.rocket().state::<JwksVerifier>() {
            Some(v) => v,
            None => {
                return Outcome::Error((Status::InternalServerError, AuthError::InternalError));
            }
        };

        match verifier.verify(token).await {
            Ok(claims) => {
                let roles = verifier.extract_roles(&claims);

                Outcome::Success(JwtGuard {
                    kc_sub: claims.sub,
                    email: claims.email,
                    username: claims.preferred_username,
                    roles,
                })
            }
            Err(e) => Outcome::Error((Status::Unauthorized, e)),
        }
    }
}
