use rocket::Request;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Keycloak error: {0}")]
    KeycloakError(String),

    #[error("User not found")]
    UserNotFound,

    #[error("User is blocked: {0}")]
    UserBlocked(String),

    #[error("Internal server error")]
    InternalError,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code identifier
    #[schema(example = "invalid_credentials")]
    pub error: String,
    /// Human-readable error message
    #[schema(example = "Invalid credentials")]
    pub message: String,
}

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let (status, error_type) = match &self {
            AuthError::InvalidCredentials => (Status::Unauthorized, "invalid_credentials"),
            AuthError::TokenExpired => (Status::Unauthorized, "token_expired"),
            AuthError::InvalidToken => (Status::Unauthorized, "invalid_token"),
            AuthError::MissingAuthHeader => (Status::Unauthorized, "missing_auth_header"),
            AuthError::KeycloakError(_) => (Status::BadGateway, "keycloak_error"),
            AuthError::UserNotFound => (Status::NotFound, "user_not_found"),
            AuthError::UserBlocked(_) => (Status::Forbidden, "user_blocked"),
            AuthError::InternalError => (Status::InternalServerError, "internal_error"),
        };

        let response = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        };

        (status, Json(response)).respond_to(req)
    }
}
