use crate::contexts::{AuthContext, User};
use crate::services::{api_client, ApiError};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use base64::{Engine as _, engine::general_purpose};

#[derive(Deserialize, Clone, Debug)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Clone, Debug)]
struct JwtClaims {
    sub: String,
    email: String,
    name: String,
    roles: Vec<String>,
    _exp: usize,
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterRequest {
    email: String,
    name: String,
    password: String,
}

#[function_component(AuthDropdown)]
pub fn auth_dropdown() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    // Prevent dropdown from closing on inside clicks
    let on_click_capture = Callback::from(|e: MouseEvent| e.stop_propagation());

    // State
    let email = use_state(String::default);
    let password = use_state(String::default);
    let name = use_state(String::default);
    let show_register = use_state(|| false);
    let message = use_state(|| None::<String>);
    let loading = use_state(|| false);

    // Submit handler (login or register based on toggle)
    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let name = name.clone();
        let show_register = show_register.clone();
        let message = message.clone();
        let loading = loading.clone();
        let auth = auth.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let email_v = (*email).clone();
            let pass_v = (*password).clone();
            let name_v = (*name).clone();
            let registering = *show_register;
            let message = message.clone();
            let loading = loading.clone();
            let auth = auth.clone();

            loading.set(true);
            message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(None);

                if registering {
                    // Register
                    let register_req = RegisterRequest {
                        email: email_v.clone(),
                        name: name_v,
                        password: pass_v.clone(),
                    };

                    match client.post::<_, serde_json::Value>("/auth/register", &register_req).await {
                        Ok(_) => {
                            // Auto-login after successful registration
                            let login_req = LoginRequest {
                                email: email_v,
                                password: pass_v,
                            };

                            match client.post::<_, LoginResponse>("/auth/login", &login_req).await {
                                Ok(resp) => {
                                    // Decode JWT to get user info
                                    if let Some(user) = decode_jwt_claims(&resp.token) {
                                        auth.login.emit((resp.token, user));
                                        if let Some(w) = web_sys::window() {
                                            let _ = w.location().reload();
                                        }
                                    } else {
                                        loading.set(false);
                                        message.set(Some("Failed to decode token".to_string()));
                                    }
                                }
                                Err(e) => {
                                    loading.set(false);
                                    message.set(Some(format!("Registered, but login failed: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            loading.set(false);
                            message.set(Some(format!("Registration failed: {}", e)));
                        }
                    }
                } else {
                    // Login
                    let login_req = LoginRequest {
                        email: email_v,
                        password: pass_v,
                    };

                    match client.post::<_, LoginResponse>("/auth/login", &login_req).await {
                        Ok(resp) => {
                            // Decode JWT to get user info
                            if let Some(user) = decode_jwt_claims(&resp.token) {
                                auth.login.emit((resp.token, user));
                                if let Some(w) = web_sys::window() {
                                    let _ = w.location().reload();
                                }
                            } else {
                                loading.set(false);
                                message.set(Some("Failed to decode token".to_string()));
                            }
                        }
                        Err(ApiError::Unauthorized) => {
                            loading.set(false);
                            message.set(Some("Invalid email or password".to_string()));
                        }
                        Err(e) => {
                            loading.set(false);
                            message.set(Some(format!("Login failed: {}", e)));
                        }
                    }
                }
            });
        })
    };

    let toggle_register = {
        let show_register = show_register.clone();
        let message = message.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            show_register.set(!*show_register);
            message.set(None);
        })
    };

    html! {
        <form onsubmit={on_submit} class="vstack gap-2" style="width: 100%;" onclick={on_click_capture}>
            if let Some(msg) = (*message).clone() {
                <div class="alert alert-warning py-1 px-2 mb-2 small">{msg}</div>
            }
            if *show_register {
                <input
                    class="form-control form-control-sm"
                    placeholder="Full name"
                    value={(*name).clone()}
                    disabled={*loading}
                    oninput={{
                        let s = name.clone();
                        Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            s.set(input.value());
                        })
                    }}
                />
            }
            <input
                class="form-control form-control-sm"
                placeholder="Email"
                type="email"
                value={(*email).clone()}
                disabled={*loading}
                oninput={{
                    let s = email.clone();
                    Callback::from(move |e: InputEvent| {
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        s.set(input.value());
                    })
                }}
            />
            <input
                type="password"
                class="form-control form-control-sm"
                placeholder="Password"
                value={(*password).clone()}
                disabled={*loading}
                oninput={{
                    let s = password.clone();
                    Callback::from(move |e: InputEvent| {
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        s.set(input.value());
                    })
                }}
            />
            <div class="d-flex justify-content-between align-items-center mt-1">
                <button class="btn btn-sm btn-primary" type="submit" disabled={*loading}>
                    if *loading {
                        <span class="spinner-border spinner-border-sm me-1" role="status" aria-hidden="true"></span>
                    }
                    { if *show_register { "Register" } else { "Login" } }
                </button>
                <button class="btn btn-sm btn-link text-decoration-none" type="button" onclick={toggle_register} disabled={*loading}>
                    { if *show_register { "Have an account? Login" } else { "Create account" } }
                </button>
            </div>
        </form>
    }
}

// Helper function to decode JWT and extract user info
fn decode_jwt_claims(token: &str) -> Option<User> {
    // JWT format: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the payload (second part)
    let payload = parts[1];

    // Add padding if needed (base64url requires it)
    let padding = match payload.len() % 4 {
        0 => "",
        2 => "==",
        3 => "=",
        _ => return None,
    };
    let padded = format!("{}{}", payload, padding);

    // Decode from base64url (replace - with + and _ with /)
    let normalized = padded.replace('-', "+").replace('_', "/");

    let decoded = match general_purpose::STANDARD.decode(&normalized) {
        Ok(d) => d,
        Err(_) => return None,
    };

    let json_str = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let claims: JwtClaims = match serde_json::from_str(&json_str) {
        Ok(c) => c,
        Err(_) => return None,
    };

    Some(User {
        id: claims.sub.parse().unwrap_or(0),
        email: claims.email,
        name: claims.name,
        roles: claims.roles,
    })
}
