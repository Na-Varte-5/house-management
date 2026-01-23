use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<User>,
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() && self.user.is_some()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.user
            .as_ref()
            .map(|u| u.roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.user
            .as_ref()
            .map(|u| roles.iter().any(|role| u.roles.iter().any(|r| r == role)))
            .unwrap_or(false)
    }

    pub fn is_admin_or_manager(&self) -> bool {
        self.has_any_role(&["Admin", "Manager"])
    }
}

#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub state: Rc<AuthState>,
    pub login: Callback<(String, User)>,
    pub logout: Callback<()>,
}

impl AuthContext {
    pub fn is_authenticated(&self) -> bool {
        self.state.is_authenticated()
    }

    pub fn user(&self) -> Option<&User> {
        self.state.user.as_ref()
    }

    pub fn token(&self) -> Option<&str> {
        self.state.token.as_deref()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.state.has_role(role)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.state.has_any_role(roles)
    }

    pub fn is_admin_or_manager(&self) -> bool {
        self.state.is_admin_or_manager()
    }
}

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let state = use_state(|| {
        // Load from localStorage on mount
        load_auth_from_storage()
    });

    let login = {
        let state = state.clone();
        Callback::from(move |(token, user): (String, User)| {
            save_auth_to_storage(&token, &user);
            state.set(AuthState {
                token: Some(token),
                user: Some(user),
            });
        })
    };

    let logout = {
        let state = state.clone();
        Callback::from(move |_| {
            clear_auth_from_storage();
            state.set(AuthState {
                token: None,
                user: None,
            });
        })
    };

    let context = AuthContext {
        state: Rc::new((*state).clone()),
        login,
        logout,
    };

    html! {
        <ContextProvider<AuthContext> context={context}>
            {props.children.clone()}
        </ContextProvider<AuthContext>>
    }
}

// Helper functions for localStorage operations

fn load_auth_from_storage() -> AuthState {
    let window = match window() {
        Some(w) => w,
        None => {
            return AuthState {
                token: None,
                user: None,
            };
        }
    };

    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => {
            return AuthState {
                token: None,
                user: None,
            };
        }
    };

    let token = storage.get_item("auth.token").ok().flatten();
    let user_json = storage.get_item("auth.user").ok().flatten();

    // Validate token expiry before loading
    if let Some(ref token_str) = token {
        if !is_token_valid(token_str) {
            // Token is expired or invalid, clear storage
            clear_auth_from_storage();
            return AuthState {
                token: None,
                user: None,
            };
        }
    }

    let user = user_json.and_then(|json| serde_json::from_str::<User>(&json).ok());

    AuthState { token, user }
}

fn save_auth_to_storage(token: &str, user: &User) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth.token", token);
            if let Ok(user_json) = serde_json::to_string(user) {
                let _ = storage.set_item("auth.user", &user_json);
            }
        }
    }
}

fn clear_auth_from_storage() {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth.token");
            let _ = storage.remove_item("auth.user");
        }
    }
}

/// Checks if a JWT token is valid (not expired)
/// Returns false if token is malformed or expired
fn is_token_valid(token: &str) -> bool {
    // JWT format: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    // Decode the payload (second part)
    let payload = parts[1];

    // Add padding if needed (base64url requires it)
    let padding = match payload.len() % 4 {
        0 => "",
        2 => "==",
        3 => "=",
        _ => return false,
    };
    let padded = format!("{}{}", payload, padding);

    // Decode from base64url (replace - with + and _ with /)
    let normalized = padded.replace('-', "+").replace('_', "/");

    let decoded = match general_purpose::STANDARD.decode(&normalized) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let payload_str = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Parse as JSON and check expiry
    #[derive(Deserialize)]
    struct TokenPayload {
        exp: u64,
    }

    let payload_data: TokenPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Check if token is expired (exp is in seconds since epoch)
    let current_time = (js_sys::Date::now() / 1000.0) as u64;
    payload_data.exp > current_time
}
