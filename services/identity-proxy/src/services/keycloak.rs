/**
 * Keycloak service module.
 * This module provides the client for communicating with Keycloak
 * for authentication, token management, and user information retrieval.
 */
use crate::config::Config;
use crate::errors::AuthError;
use crate::models::{TokenResponse, UserInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/**
 * Internal token request structure.
 * Used to build OAuth2 token requests to Keycloak.
 */
#[derive(Debug, Serialize)]
struct TokenRequest<'a> {
    grant_type: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<&'a str>,
}

/**
 * Internal Keycloak error response structure.
 * Used to parse error responses from Keycloak API.
 */
#[derive(Debug, Deserialize)]
struct KeycloakError {
    error: String,
    error_description: Option<String>,
}

/**
 * Keycloak service client.
 * Handles all communication with the Keycloak server for authentication operations.
 */
pub struct KeycloakService {
    pub config: Config,
    client: Client,
}

impl KeycloakService {
    /**
     * Creates a new Keycloak service instance.
     * Initializes the HTTP client with a 30-second timeout.
     *
     * # Arguments
     * * `config` - Keycloak configuration containing URLs and credentials
     *
     * # Returns
     * * `KeycloakService` - Configured service instance
     */
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /**
     * Authenticates a user with username and password.
     * Uses the OAuth2 Resource Owner Password Credentials grant.
     *
     * # Arguments
     * * `username` - User's email or username
     * * `password` - User's password
     *
     * # Returns
     * * `Ok(TokenResponse)` - JWT tokens on successful authentication
     * * `Err(AuthError)` - Error if authentication fails
     */
    pub async fn login(&self, username: &str, password: &str) -> Result<TokenResponse, AuthError> {
        let request = TokenRequest {
            grant_type: "password",
            client_id: &self.config.keycloak_client_id,
            client_secret: &self.config.keycloak_client_secret,
            username: Some(username),
            password: Some(password),
            refresh_token: None,
            code: None,
            redirect_uri: None,
        };

        self.request_token(request).await
    }

    /**
     * Refreshes an expired access token using a valid refresh token.
     * Returns new JWT tokens without requiring re-authentication.
     *
     * # Arguments
     * * `refresh_token` - Valid refresh token from previous authentication
     *
     * # Returns
     * * `Ok(TokenResponse)` - New JWT tokens on success
     * * `Err(AuthError)` - Error if refresh token is invalid or expired
     */
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AuthError> {
        let request = TokenRequest {
            grant_type: "refresh_token",
            client_id: &self.config.keycloak_client_id,
            client_secret: &self.config.keycloak_client_secret,
            username: None,
            password: None,
            refresh_token: Some(refresh_token),
            code: None,
            redirect_uri: None,
        };

        self.request_token(request).await
    }

    /**
     * Exchanges an authorization code for tokens.
     * Used in the OAuth2 Authorization Code Flow after user redirects back from Keycloak.
     *
     * # Arguments
     * * `code` - Authorization code received from Keycloak callback
     *
     * # Returns
     * * `Ok(TokenResponse)` - JWT tokens on success
     * * `Err(AuthError)` - Error if code is invalid
     */
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse, AuthError> {
        let request = TokenRequest {
            grant_type: "authorization_code",
            client_id: &self.config.keycloak_client_id,
            client_secret: &self.config.keycloak_client_secret,
            username: None,
            password: None,
            refresh_token: None,
            code: Some(code),
            redirect_uri: Some(&self.config.redirect_uri),
        };

        self.request_token(request).await
    }

    /**
     * Generates the Keycloak authorization URL for OAuth2 login.
     * Redirects user to Keycloak login page.
     *
     * # Arguments
     * * `state` - Optional state parameter for CSRF protection
     *
     * # Returns
     * * `String` - Full authorization URL to redirect user to
     */
    pub fn get_authorization_url(&self, state: Option<&str>) -> String {
        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email",
            self.config.authorization_url(),
            self.config.keycloak_client_id,
            urlencoding::encode(&self.config.redirect_uri)
        );

        if let Some(s) = state {
            url.push_str(&format!("&state={}", urlencoding::encode(s)));
        }

        url
    }

    /**
     * Returns the frontend URL for redirecting after authentication.
     */
    pub fn get_frontend_url(&self) -> &str {
        &self.config.frontend_url
    }

    /**
     * Internal method to send token requests to Keycloak.
     * Handles the HTTP communication and error parsing.
     *
     * # Arguments
     * * `request` - Token request payload
     *
     * # Returns
     * * `Ok(TokenResponse)` - Tokens on success
     * * `Err(AuthError)` - Parsed error on failure
     */
    async fn request_token(&self, request: TokenRequest<'_>) -> Result<TokenResponse, AuthError> {
        let response = self
            .client
            .post(self.config.token_url())
            .form(&request)
            .send()
            .await
            .map_err(|e| AuthError::KeycloakError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<TokenResponse>()
                .await
                .map_err(|e| AuthError::KeycloakError(e.to_string()))
        } else {
            let error: KeycloakError = response.json().await.unwrap_or(KeycloakError {
                error: "unknown".to_string(),
                error_description: None,
            });

            match error.error.as_str() {
                "invalid_grant" => Err(AuthError::InvalidCredentials),
                "invalid_token" => Err(AuthError::TokenExpired),
                _ => Err(AuthError::KeycloakError(
                    error.error_description.unwrap_or(error.error),
                )),
            }
        }
    }

    /**
     * Retrieves user information from Keycloak's userinfo endpoint.
     * Requires a valid access token.
     *
     * # Arguments
     * * `access_token` - Valid JWT access token
     *
     * # Returns
     * * `Ok(UserInfo)` - User profile information
     * * `Err(AuthError)` - Error if token is invalid
     */
    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, AuthError> {
        let response = self
            .client
            .get(self.config.userinfo_url())
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::KeycloakError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<UserInfo>()
                .await
                .map_err(|e| AuthError::KeycloakError(e.to_string()))
        } else {
            Err(AuthError::InvalidToken)
        }
    }

    /**
     * Logs out a user by invalidating their refresh token.
     * Terminates the user session on Keycloak server.
     *
     * # Arguments
     * * `refresh_token` - Refresh token to invalidate
     *
     * # Returns
     * * `Ok(())` - Success on logout
     * * `Err(AuthError)` - Error if logout fails
     */
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        let params = [
            ("client_id", self.config.keycloak_client_id.as_str()),
            ("client_secret", self.config.keycloak_client_secret.as_str()),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .client
            .post(self.config.logout_url())
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::KeycloakError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AuthError::KeycloakError("Logout failed".to_string()))
        }
    }
}
