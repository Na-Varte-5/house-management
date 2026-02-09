use crate::auth::error::AppError;
use crate::auth::types::{Claims, JwtKeys};
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{Algorithm, Validation, decode};
use std::future::{Ready, ready};

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub claims: Claims,
}

impl AuthContext {
    pub fn user_id(&self) -> Result<u64, AppError> {
        self.claims
            .sub
            .parse::<u64>()
            .map_err(|_| AppError::Unauthorized)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        crate::auth::roles::has_any_role(&self.claims.roles, roles)
    }
    pub fn require_roles(&self, roles: &[&str]) -> Result<(), AppError> {
        if self.has_any_role(roles) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl FromRequest for AuthContext {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let keys = match req.app_data::<actix_web::web::Data<JwtKeys>>() {
            Some(k) => k,
            None => return ready(Err(AppError::Internal("missing_keys".into()))),
        };
        let auth_header = match req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
        {
            Some(v) => v,
            None => return ready(Err(AppError::Unauthorized)),
        };
        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) => t,
            None => return ready(Err(AppError::Unauthorized)),
        };
        let data = match decode::<Claims>(token, &keys.decoding, &Validation::new(Algorithm::HS256))
        {
            Ok(d) => d,
            Err(_e) => return ready(Err(AppError::Token)),
        };
        ready(Ok(AuthContext {
            claims: data.claims,
        }))
    }
}
