/**
 * Proxy module for API Gateway.
 * This module handles forwarding requests to downstream microservices.
 */
use crate::config::Config;
use crate::errors::GatewayError;
use reqwest::Client;
use rocket::Request;
use rocket::http::{ContentType, Status};
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

pub struct ProxyClient {
    client: Client,
    config: Config,
}

/**
 * Proxy response wrapper.
 * Wraps the response from downstream services for Rocket.
 */
pub struct ProxyResponse {
    pub status: Status,
    pub content_type: ContentType,
    pub body: Vec<u8>,
}

impl<'r> Responder<'r, 'static> for ProxyResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(self.status)
            .header(self.content_type)
            .sized_body(self.body.len(), Cursor::new(self.body))
            .ok()
    }
}

impl ProxyClient {
    /**
     * Creates a new proxy client with the given configuration.
     *
     * # Arguments
     * * `config` - Gateway configuration
     *
     * # Returns
     * * `ProxyClient` - Configured proxy client
     */
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /**
     * Forwards a GET request to the identity-proxy service.
     *
     * # Arguments
     * * `path` - The path to forward (without base URL)
     * * `auth_header` - Optional authorization header
     *
     * # Returns
     * * `Ok(ProxyResponse)` - Response from downstream service
     * * `Err(GatewayError)` - Error if request fails
     */
    #[allow(dead_code)]
    pub async fn forward_get(
        &self,
        path: &str,
        auth_header: Option<&str>,
    ) -> Result<ProxyResponse, GatewayError> {
        let url = format!("{}{}", self.config.identity_proxy_url, path);

        let mut request = self.client.get(&url);

        if let Some(auth) = auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| GatewayError::ServiceUnavailable(e.to_string()))?;

        let status = Status::from_code(response.status().as_u16()).unwrap_or(Status::BadGateway);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(ContentType::parse_flexible)
            .unwrap_or(ContentType::JSON);

        let body = response
            .bytes()
            .await
            .map_err(|e| GatewayError::BadGateway(e.to_string()))?
            .to_vec();

        Ok(ProxyResponse {
            status,
            content_type,
            body,
        })
    }

    /**
     * Forwards a POST request to the identity-proxy service.
     *
     * # Arguments
     * * `path` - The path to forward (without base URL)
     * * `body` - Request body as bytes
     * * `auth_header` - Optional authorization header
     *
     * # Returns
     * * `Ok(ProxyResponse)` - Response from downstream service
     * * `Err(GatewayError)` - Error if request fails
     */
    pub async fn forward_post(
        &self,
        path: &str,
        body: Vec<u8>,
        auth_header: Option<&str>,
    ) -> Result<ProxyResponse, GatewayError> {
        let url = format!("{}{}", self.config.identity_proxy_url, path);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body);

        if let Some(auth) = auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request
            .send()
            .await
            .map_err(|e| GatewayError::ServiceUnavailable(e.to_string()))?;

        let status = Status::from_code(response.status().as_u16()).unwrap_or(Status::BadGateway);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(ContentType::parse_flexible)
            .unwrap_or(ContentType::JSON);

        let body = response
            .bytes()
            .await
            .map_err(|e| GatewayError::BadGateway(e.to_string()))?
            .to_vec();

        Ok(ProxyResponse {
            status,
            content_type,
            body,
        })
    }

    /**
     * Checks the health of the identity-proxy service.
     *
     * # Returns
     * * `true` if service is healthy
     * * `false` if service is unavailable
     */
    pub async fn check_identity_proxy_health(&self) -> bool {
        let url = format!("{}/api/v1/health", self.config.identity_proxy_url);

        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /**
     * Returns the identity-proxy base URL.
     */
    pub fn get_identity_proxy_url(&self) -> &str {
        &self.config.identity_proxy_url
    }
}
