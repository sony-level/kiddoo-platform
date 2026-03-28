/**
 * Configuration module for API Gateway.
 * This module handles environment-based configuration for service discovery
 * and routing to downstream microservices.
 */
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[allow(dead_code)]
    pub gateway_port: u16,
    pub identity_proxy_url: String,
}

impl Config {
    /**
     * Loads configuration from environment variables.
     * Falls back to default values for local development.
     *
     * # Returns
     * * `Config` - Gateway configuration
     */
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            gateway_port: env::var("GATEWAY_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            identity_proxy_url: env::var("IDENTITY_PROXY_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
        }
    }
}
