use crate::auth::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand::thread_rng;
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|p| p.to_string())
        .map_err(|e| AppError::Crypto(e.to_string()))
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};
    if let Ok(parsed) = PasswordHash::new(password_hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let pwd = "S3cret!";
        let hashed = hash_password(pwd).expect("hash ok");
        assert!(verify_password(pwd, &hashed));
        assert!(!verify_password("wrong", &hashed));
    }

    #[test]
    fn verify_invalid_hash() {
        assert!(!verify_password("anything", "not-a-valid-hash"));
    }
}
