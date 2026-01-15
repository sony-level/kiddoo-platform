use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,
    pub jwt_secret: String,
    pub redirect_uri: String,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            keycloak_url: env::var("KEYCLOAK_URL")?,
            keycloak_realm: env::var("KEYCLOAK_REALM")?,
            keycloak_client_id: env::var("KEYCLOAK_CLIENT_ID")?,
            keycloak_client_secret: env::var("KEYCLOAK_CLIENT_SECRET")?,
            jwt_secret: env::var("JWT_SECRET")?,
            redirect_uri: env::var("REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8000/api/v1/auth/callback".to_string()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }

    pub fn issuer_url(&self) -> String {
        format!("{}/realms/{}", self.keycloak_url, self.keycloak_realm)
    }

    pub fn authorization_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/auth",
            self.keycloak_url, self.keycloak_realm
        )
    }

    pub fn token_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.keycloak_url, self.keycloak_realm
        )
    }

    pub fn userinfo_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/userinfo",
            self.keycloak_url, self.keycloak_realm
        )
    }

    pub fn logout_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/logout",
            self.keycloak_url, self.keycloak_realm
        )
    }
}
