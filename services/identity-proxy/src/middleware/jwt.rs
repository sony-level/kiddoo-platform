/**
 * JWT middleware module.
 * This module provides request guards for JWT token validation and
 * extraction of authenticated user information from requests.
 */
use crate::errors::AuthError;
use crate::models::Claims;
use jsonwebtoken::{DecodingKey, Validation, decode};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use uuid::Uuid;

/**
 * JWT request guard for protected routes.
 * Automatically validates JWT tokens and extracts user information
 * from the Authorization header.
 */
#[derive(Debug)]
pub struct JwtGuard {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub roles: Vec<String>,
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

        let jwt_secret = match std::env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                return Outcome::Error((Status::InternalServerError, AuthError::InternalError));
            }
        };

        let validation = Validation::default();
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());

        match decode::<Claims>(token, &key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;

                let user_id = match Uuid::parse_str(&claims.sub) {
                    Ok(id) => id,
                    Err(_) => {
                        return Outcome::Error((Status::Unauthorized, AuthError::InvalidToken));
                    }
                };

                let roles = claims.realm_access.map(|ra| ra.roles).unwrap_or_default();

                Outcome::Success(JwtGuard {
                    user_id,
                    email: claims.email,
                    username: claims.preferred_username,
                    roles,
                })
            }
            Err(e) => {
                let error = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                };
                Outcome::Error((Status::Unauthorized, error))
            }
        }
    }
}
