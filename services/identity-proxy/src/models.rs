/**
 * Models module for identity-proxy service.
 * This module contains all data transfer objects (DTOs) and response structures
 * used across the authentication service.
 */
use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/**
 * Request payload for user login.
 * Contains the credentials required to authenticate a user via Keycloak.
 */
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "password": "password123"}))]
pub struct LoginRequest {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

/**
 * Response payload after successful authentication.
 * Contains JWT tokens and metadata for client-side session management.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT access token for API authentication
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Refresh token to obtain new access tokens
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    /// Token expiration time in seconds
    #[schema(example = 300)]
    pub expires_in: i64,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
}

/**
 * Request payload for token refresh.
 * Used to obtain a new access token using a valid refresh token.
 */
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}))]
pub struct RefreshRequest {
    /// Valid refresh token from previous authentication
    pub refresh_token: String,
}

/**
 * Request payload for user logout.
 * Contains the refresh token to invalidate the user session.
 */
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}))]
pub struct LogoutRequest {
    /// Refresh token to invalidate
    pub refresh_token: String,
}

/**
 * Generic message response.
 * Used for simple success/info responses without complex data.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    /// Response message
    #[schema(example = "Successfully logged out")]
    pub message: String,
}

/**
 * User information response.
 * Contains the authenticated user's profile data extracted from JWT claims.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    /// User unique identifier (UUID)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: Option<String>,
    /// Username
    #[schema(example = "johndoe")]
    pub username: Option<String>,
    /// User roles
    #[schema(example = json!(["parent", "verified"]))]
    pub roles: Vec<String>,
}

/**
 * Health check response.
 * Provides service status information for monitoring and load balancing.
 */
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service health status
    #[schema(example = "ok")]
    pub status: String,
    /// Service name
    #[schema(example = "identity-proxy")]
    pub service: String,
    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,
}

/**
 * JWT Claims structure.
 * Represents the decoded payload from a Keycloak-issued JWT token.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: Option<String>,
    pub aud: Option<AudClaim>,
    pub exp: usize,
    pub iat: usize,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, ResourceAccess>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudClaim {
    Single(String),
    Multiple(Vec<String>),
}

/**
 * Keycloak realm access structure.
 * Contains the user's roles within the Keycloak realm.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub roles: Vec<String>,
}

/**
 * Keycloak token response.
 * Raw response from Keycloak's token endpoint after successful authentication.
 */
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub refresh_token: String,
    pub token_type: String,
    #[serde(default)]
    pub scope: String,
}

/**
 * Keycloak user info response.
 * Contains user profile information from Keycloak's userinfo endpoint.
 */
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub preferred_username: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

/**
 * OAuth2 callback query parameters.
 * Received from Keycloak after user authentication.
 */
#[derive(Debug, Deserialize, FromForm)]
pub struct AuthCallback {
    /// Authorization code from Keycloak
    pub code: String,
    /// State parameter for CSRF protection
    pub state: Option<String>,
}
