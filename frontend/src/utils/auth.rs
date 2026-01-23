// LEGACY AUTH ADAPTER
// This module is deprecated but kept for backwards compatibility with old components.
// New code should use AuthContext from contexts::auth instead.
//
// TODO: Migrate comment_list.rs and announcement_list.rs to use AuthContext and api_client,
// then remove this file.

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

/// DEPRECATED: Use AuthContext.user() instead
pub fn current_user() -> Option<DecodedClaims> {
    get_token().and_then(|t| decode_token(&t))
}

/// DEPRECATED: Use AuthContext.token() instead
pub fn get_token() -> Option<String> {
    let window: Window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    // Try new auth system first (auth.token), fallback to old (jwt)
    storage
        .get_item("auth.token")
        .ok()
        .flatten()
        .or_else(|| storage.get_item("jwt").ok().flatten())
}

/// DEPRECATED: Use AuthContext.login() instead
pub fn set_token(tok: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            // Store in new location for compatibility
            let _ = storage.set_item("auth.token", tok);
        }
    }
}

/// DEPRECATED: Use AuthContext.logout() instead
pub fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth.token");
            let _ = storage.remove_item("auth.user");
            let _ = storage.remove_item("jwt"); // clear old location too
        }
    }
}

fn decode_token(token: &str) -> Option<DecodedClaims> {
    use base64::Engine;
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
