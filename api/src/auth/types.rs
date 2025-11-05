use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}
impl JwtKeys {
    pub fn from_secret(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String, // added user name for UI display
    pub roles: Vec<String>,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_serialize_roundtrip() {
        let c = Claims {
            sub: "1".into(),
            email: "user@example.com".into(),
            name: "Test User".into(),
            roles: vec!["Admin".into(), "Homeowner".into()],
            exp: 123456,
        };
        let json = serde_json::to_string(&c).unwrap();
        let de: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "Test User");
        assert_eq!(de.roles.len(), 2);
    }
}
