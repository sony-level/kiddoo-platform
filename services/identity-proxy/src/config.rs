use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,
    pub kc_issuer: String,
    pub kc_jwks_url: String,
    pub kc_audience: String,
    pub kc_client_id: String,
    pub jwks_cache_ttl_seconds: u64,
    pub redirect_uri: String,
    pub frontend_url: String,
    pub oauth_state: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        let oauth_state = env::var("OAUTH_STATE")?;
        let keycloak_url = env::var("KEYCLOAK_URL")?;
        let keycloak_realm = env::var("KEYCLOAK_REALM")?;
        let keycloak_client_id = env::var("KEYCLOAK_CLIENT_ID")?;

        let issuer_url = format!("{}/realms/{}", keycloak_url, keycloak_realm);
        let jwks_url = format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            keycloak_url, keycloak_realm
        );

        Ok(Self {
            keycloak_url,
            keycloak_realm,
            keycloak_client_id: keycloak_client_id.clone(),
            keycloak_client_secret: env::var("KEYCLOAK_CLIENT_SECRET")?,
            kc_issuer: env::var("KC_ISSUER").unwrap_or(issuer_url),
            kc_jwks_url: env::var("KC_JWKS_URL").unwrap_or(jwks_url),
            kc_audience: env::var("KC_AUDIENCE").unwrap_or_else(|_| keycloak_client_id.clone()),
            kc_client_id: env::var("KC_CLIENT_ID").unwrap_or(keycloak_client_id),
            jwks_cache_ttl_seconds: env::var("JWKS_CACHE_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(300),
            redirect_uri: env::var("REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8000/api/v1/auth/callback".to_string()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            oauth_state,
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
