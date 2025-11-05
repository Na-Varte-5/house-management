use base64::Engine; // bring decode method into scope
use serde::Deserialize;
use web_sys::Window;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecodedClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

pub fn decode_token(token: &str) -> Option<DecodedClaims> {
    // JWT format: header.payload.signature (base64url)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload_b64 = parts[1];
    // base64url decode
    let mut s = payload_b64.replace('-', "+").replace('_', "/");
    // pad
    while s.len() % 4 != 0 {
        s.push('=');
    }
    let decoded = base64::engine::general_purpose::STANDARD.decode(&s).ok()?;
    serde_json::from_slice::<DecodedClaims>(&decoded).ok()
}

pub fn current_user() -> Option<DecodedClaims> {
    get_token().and_then(|t| decode_token(&t))
}

pub fn get_token() -> Option<String> {
    let window: Window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("jwt").ok().flatten()
}

pub fn set_token(tok: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("jwt", tok);
        }
    }
}

pub fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("jwt");
        }
    }
}
