use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

impl SecurityConfig {
    pub fn new(secret: String, expiry_hours: i64) -> Self {
        Self {
            jwt_secret: secret,
            jwt_expiry_hours: expiry_hours,
        }
    }

    pub fn default_with_secret(secret: String) -> Self {
        Self {
            jwt_secret: secret,
            jwt_expiry_hours: 24,
        }
    }
}

pub struct JwtManager {
    config: Arc<SecurityConfig>,
}

impl JwtManager {
    pub fn new(config: Arc<SecurityConfig>) -> Self {
        Self { config }
    }

    pub fn generate_token(&self, user_id: &str, role: &str) -> Result<String, String> {
        let now = chrono::Utc::now().timestamp();
        let expiry = now + (self.config.jwt_expiry_hours * 3600);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry,
            iat: now,
            role: role.to_string(),
        };

        let key = EncodingKey::from_secret(self.config.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &key).map_err(|e| e.to_string())
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let key = DecodingKey::from_secret(self.config.jwt_secret.as_bytes());
        decode::<Claims>(token, &key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| e.to_string())
    }

    pub fn hash_password(&self, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(self.config.jwt_secret.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }
}

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format".to_string(),
        ));
    }

    let token = &auth_header[7..];
    debug!("Validating JWT token");

    // Token validation would happen here with JwtManager
    // For now, we just check if token is present
    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty token".to_string()));
    }

    Ok(next.run(req).await)
}

pub fn validate_input(input: &str, max_length: usize) -> Result<(), String> {
    if input.is_empty() {
        return Err("Input cannot be empty".to_string());
    }

    if input.len() > max_length {
        return Err(format!("Input exceeds maximum length of {}", max_length));
    }

    // Check for SQL injection patterns
    let dangerous_patterns = ["--", "/*", "*/", "xp_", "sp_", "exec", "execute"];
    for pattern in &dangerous_patterns {
        if input.to_lowercase().contains(pattern) {
            return Err("Input contains potentially dangerous patterns".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_verification() {
        let config = Arc::new(SecurityConfig::new("test_secret".to_string(), 24));
        let manager = JwtManager::new(config);

        let token = manager.generate_token("user123", "admin").unwrap();
        let claims = manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_password_hashing() {
        let config = Arc::new(SecurityConfig::new("test_secret".to_string(), 24));
        let manager = JwtManager::new(config);

        let password = "secure_password";
        let hash = manager.hash_password(password);
        assert!(manager.verify_password(password, &hash));
        assert!(!manager.verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_input_validation() {
        assert!(validate_input("valid_input", 100).is_ok());
        assert!(validate_input("", 100).is_err());
        assert!(validate_input("a".repeat(101), 100).is_err());
        assert!(validate_input("DROP TABLE users--", 100).is_err());
    }
}
