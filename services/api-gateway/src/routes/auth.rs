/**
 * Authentication proxy routes for API Gateway.
 * This module forwards authentication requests to the identity-proxy service.
 */
use crate::errors::{ErrorResponse, GatewayError};
use crate::models::{
    LoginRequest, LoginResponse, LogoutRequest, MessageResponse, RefreshRequest, UserResponse,
};
use crate::proxy::{ProxyClient, ProxyResponse};
use rocket::State;
use rocket::data::{Data, ToByteUnit};
use rocket::request::Request;
use rocket::response::Redirect;
use rocket::{get, post};

/**
 * Initiates OAuth2 Authorization Code Flow.
 * Redirects to identity-proxy which then redirects to Keycloak login page.
 */
#[utoipa::path(
    get,
    path = "/auth/authorize",
    tag = "auth",
    responses(
        (status = 302, description = "Redirect to Keycloak login page")
    )
)]
#[get("/auth/authorize")]
pub async fn authorize(proxy: &State<ProxyClient>) -> Result<Redirect, GatewayError> {
    // Get the redirect URL from identity-proxy
    let url = format!(
        "{}/api/v1/auth/authorize",
        proxy.inner().get_identity_proxy_url()
    );
    Ok(Redirect::to(url))
}

/**
 * OAuth2 callback endpoint.
 * Forwards callback from Keycloak to identity-proxy.
 */
#[utoipa::path(
    get,
    path = "/auth/callback",
    tag = "auth",
    params(
        ("code" = String, Query, description = "Authorization code from Keycloak"),
        ("state" = Option<String>, Query, description = "State parameter for CSRF protection")
    ),
    responses(
        (status = 302, description = "Redirect to frontend with tokens"),
        (status = 401, description = "Invalid authorization code", body = ErrorResponse)
    )
)]
#[get("/auth/callback?<code>&<state>")]
pub async fn callback(
    proxy: &State<ProxyClient>,
    code: String,
    state: Option<String>,
) -> Result<Redirect, GatewayError> {
    let mut url = format!(
        "{}/api/v1/auth/callback?code={}",
        proxy.inner().get_identity_proxy_url(),
        code
    );
    if let Some(s) = state {
        url.push_str(&format!("&state={}", s));
    }
    Ok(Redirect::to(url))
}

/**
 * Forwards login request to identity-proxy (Direct Access Grant).
 */
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully authenticated", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 502, description = "Identity service unavailable", body = ErrorResponse)
    )
)]
#[post("/auth/login", data = "<body>")]
pub async fn login(
    proxy: &State<ProxyClient>,
    body: Data<'_>,
) -> Result<ProxyResponse, GatewayError> {
    let body_bytes = body
        .open(1.mebibytes())
        .into_bytes()
        .await
        .map_err(|_| GatewayError::InternalError)?
        .to_vec();

    proxy
        .forward_post("/api/v1/auth/login", body_bytes, None)
        .await
}

/**
 * Forwards refresh token request to identity-proxy.
 */
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 502, description = "Identity service unavailable", body = ErrorResponse)
    )
)]
#[post("/auth/refresh", data = "<body>")]
pub async fn refresh(
    proxy: &State<ProxyClient>,
    body: Data<'_>,
) -> Result<ProxyResponse, GatewayError> {
    let body_bytes = body
        .open(1.mebibytes())
        .into_bytes()
        .await
        .map_err(|_| GatewayError::InternalError)?
        .to_vec();

    proxy
        .forward_post("/api/v1/auth/refresh", body_bytes, None)
        .await
}

/**
 * Forwards logout request to identity-proxy.
 */
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Successfully logged out", body = MessageResponse),
        (status = 502, description = "Identity service unavailable", body = ErrorResponse)
    )
)]
#[post("/auth/logout", data = "<body>")]
pub async fn logout(
    proxy: &State<ProxyClient>,
    body: Data<'_>,
) -> Result<ProxyResponse, GatewayError> {
    let body_bytes = body
        .open(1.mebibytes())
        .into_bytes()
        .await
        .map_err(|_| GatewayError::InternalError)?
        .to_vec();

    proxy
        .forward_post("/api/v1/auth/logout", body_bytes, None)
        .await
}

/**
 * Authorization header extractor.
 */
pub struct AuthHeader(pub Option<String>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthHeader {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let auth = request
            .headers()
            .get_one("Authorization")
            .map(|s| s.to_string());
        rocket::request::Outcome::Success(AuthHeader(auth))
    }
}

/**
 * Forwards user profile request to identity-proxy.
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
pub async fn me(
    proxy: &State<ProxyClient>,
    auth: AuthHeader,
) -> Result<ProxyResponse, GatewayError> {
    proxy
        .forward_post("/api/v1/auth/me", vec![], auth.0.as_deref())
        .await
}
