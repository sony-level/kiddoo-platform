/**
 * Authentication routes module.
 * This module handles all authentication-related HTTP endpoints including
 * login, logout, token refresh, and user profile retrieval.
 *
 * ## API Endpoints
 *
 * | Method | Endpoint          | Security    | Description                    |
 * |--------|-------------------|-------------|--------------------------------|
 * | POST   | /auth/login       | Public      | Authenticate user              |
 * | POST   | /auth/refresh     | Public      | Refresh access token           |
 * | POST   | /auth/logout      | Public      | Invalidate refresh token       |
 * | POST   | /auth/me          |   JWT      | Get authenticated user profile |
 */
use crate::errors::{AuthError, ErrorResponse};
use crate::middleware::JwtGuard;
use crate::models::{
    LoginRequest, LoginResponse, LogoutRequest, MessageResponse, RefreshRequest, UserResponse,
};
use crate::services::KeycloakService;
use rocket::serde::json::Json;
use rocket::{State, post};

/**
 * Authenticates a user with email and password.
 * Sends credentials to Keycloak and returns JWT tokens on success.
 */
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully authenticated", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 502, description = "Keycloak server error", body = ErrorResponse)
    )
)]
#[post("/auth/login", data = "<request>")]
pub async fn login(
    keycloak: &State<KeycloakService>,
    request: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let tokens = keycloak.login(&request.email, &request.password).await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        token_type: tokens.token_type,
    }))
}

/**
 * Refreshes an expired access token using a valid refresh token.
 * Returns new JWT tokens without requiring re-authentication. *
 */
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 502, description = "Keycloak server error", body = ErrorResponse)
    )
)]
#[post("/auth/refresh", data = "<request>")]
pub async fn refresh(
    keycloak: &State<KeycloakService>,
    request: Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let tokens = keycloak.refresh_token(&request.refresh_token).await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        token_type: tokens.token_type,
    }))
}

/**
 * Logs out a user by invalidating their refresh token.
 * Terminates the user session on Keycloak server.
 */
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Successfully logged out", body = MessageResponse),
        (status = 502, description = "Keycloak server error", body = ErrorResponse)
    )
)]
#[post("/auth/logout", data = "<request>")]
pub async fn logout(
    keycloak: &State<KeycloakService>,
    request: Json<LogoutRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    keycloak.logout(&request.refresh_token).await?;

    Ok(Json(MessageResponse {
        message: "Successfully logged out".to_string(),
    }))
}

/**
 * Retrieves the authenticated user's profile information.
 * Extracts user data from the JWT token claims.
 */
#[utoipa::path(
    post,
    path = "/auth/me",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "User profile retrieved", body = UserResponse),
        (status = 401, description = "Missing or invalid JWT token", body = ErrorResponse)
    )
)]
#[post("/auth/me")]
pub async fn me(user: JwtGuard) -> Json<UserResponse> {
    Json(UserResponse {
        id: user.user_id.to_string(),
        email: user.email,
        username: user.username,
        roles: user.roles,
    })
}
