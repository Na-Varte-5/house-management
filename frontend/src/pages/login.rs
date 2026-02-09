use crate::contexts::{AuthContext, User};
use crate::i18n::t;
use crate::services::{ApiError, api_client};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Deserialize, Clone)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize, Clone)]
struct JwtClaims {
    sub: String,
    email: String,
    name: String,
    roles: Vec<String>,
    #[allow(dead_code)]
    exp: usize,
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

fn decode_jwt_claims(token: &str) -> Option<User> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = parts[1];
    let padding = match payload.len() % 4 {
        0 => "",
        2 => "==",
        3 => "=",
        _ => return None,
    };
    let padded = format!("{}{}", payload, padding);
    let normalized = padded.replace('-', "+").replace('_', "/");
    let decoded = general_purpose::STANDARD.decode(&normalized).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    let claims: JwtClaims = serde_json::from_str(&json_str).ok()?;
    Some(User {
        id: claims.sub.parse().unwrap_or(0),
        email: claims.email,
        name: claims.name,
        roles: claims.roles,
    })
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let show_register = use_state(|| false);
    let email = use_state(String::default);
    let password = use_state(String::default);
    let name = use_state(String::default);
    let message = use_state(|| None::<(String, bool)>);
    let loading = use_state(|| false);

    if auth.is_authenticated() {
        return html! {
            <div class="container" style="max-width: 480px; padding-top: 100px;">
                <div class="card shadow-sm">
                    <div class="card-body text-center py-4">
                        <i class="bi bi-check-circle text-success" style="font-size: 3rem;"></i>
                        <h5 class="mt-3">{t("login-already-signed-in")}</h5>
                        if let Some(user) = auth.user() {
                            <p class="text-muted">{format!("Welcome, {}!", user.name)}</p>
                        }
                    </div>
                </div>
            </div>
        };
    }

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
                    let register_req = RegisterRequest {
                        email: email_v.clone(),
                        name: name_v,
                        password: pass_v.clone(),
                    };

                    match client
                        .post::<_, serde_json::Value>("/auth/register", &register_req)
                        .await
                    {
                        Ok(_) => {
                            let login_req = LoginRequest {
                                email: email_v,
                                password: pass_v,
                            };
                            match client
                                .post::<_, LoginResponse>("/auth/login", &login_req)
                                .await
                            {
                                Ok(resp) => {
                                    if let Some(user) = decode_jwt_claims(&resp.token) {
                                        auth.login.emit((resp.token, user));
                                        if let Some(w) = web_sys::window() {
                                            let _ = w.location().set_href("/");
                                        }
                                    } else {
                                        loading.set(false);
                                        message.set(Some((t("login-failed-decode-token"), true)));
                                    }
                                }
                                Err(e) => {
                                    loading.set(false);
                                    message.set(Some((
                                        format!("{}: {}", t("login-register-then-login-failed"), e),
                                        true,
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            loading.set(false);
                            message.set(Some((
                                format!("{}: {}", t("login-register-failed"), e),
                                true,
                            )));
                        }
                    }
                } else {
                    let login_req = LoginRequest {
                        email: email_v,
                        password: pass_v,
                    };
                    match client
                        .post::<_, LoginResponse>("/auth/login", &login_req)
                        .await
                    {
                        Ok(resp) => {
                            if let Some(user) = decode_jwt_claims(&resp.token) {
                                auth.login.emit((resp.token, user));
                                if let Some(w) = web_sys::window() {
                                    let _ = w.location().set_href("/");
                                }
                            } else {
                                loading.set(false);
                                message.set(Some((t("login-failed-decode-token"), true)));
                            }
                        }
                        Err(ApiError::Unauthorized) => {
                            loading.set(false);
                            message.set(Some((t("login-invalid-credentials"), true)));
                        }
                        Err(e) => {
                            loading.set(false);
                            message.set(Some((format!("{}: {}", t("login-failed"), e), true)));
                        }
                    }
                }
            });
        })
    };

    let toggle_register = {
        let show_register = show_register.clone();
        let message = message.clone();
        Callback::from(move |_: MouseEvent| {
            show_register.set(!*show_register);
            message.set(None);
        })
    };

    let on_email = {
        let s = email.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            s.set(input.value());
        })
    };
    let on_password = {
        let s = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            s.set(input.value());
        })
    };
    let on_name = {
        let s = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            s.set(input.value());
        })
    };

    let is_register = *show_register;

    html! {
        <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh; background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);">
            <div class="card shadow" style="width: 100%; max-width: 420px;">
                <div class="card-body p-4">
                    <div class="text-center mb-4">
                        <span style="font-size: 2.5rem;">{"🏠"}</span>
                        <h4 class="mt-2 mb-0">{t("login-title")}</h4>
                        <p class="text-muted small">
                            {if is_register { t("login-create-account") } else { t("login-sign-in-to-continue") }}
                        </p>
                    </div>

                    <ul class="nav nav-pills nav-justified mb-3">
                        <li class="nav-item">
                            <button
                                class={classes!("nav-link", (!is_register).then_some("active"))}
                                onclick={toggle_register.clone()}
                                disabled={*loading}
                            >
                                {t("login-sign-in")}
                            </button>
                        </li>
                        <li class="nav-item">
                            <button
                                class={classes!("nav-link", is_register.then_some("active"))}
                                onclick={toggle_register.clone()}
                                disabled={*loading}
                            >
                                {t("login-register")}
                            </button>
                        </li>
                    </ul>

                    if let Some((msg, is_error)) = (*message).clone() {
                        <div class={classes!("alert", "py-2", "small", if is_error { "alert-danger" } else { "alert-success" })}>
                            {msg}
                        </div>
                    }

                    <form onsubmit={on_submit}>
                        if is_register {
                            <div class="mb-3">
                                <label class="form-label" for="login-name">{t("label-full-name")}</label>
                                <input
                                    id="login-name"
                                    class="form-control"
                                    placeholder="John Doe"
                                    value={(*name).clone()}
                                    disabled={*loading}
                                    oninput={on_name}
                                    required=true
                                />
                            </div>
                        }
                        <div class="mb-3">
                            <label class="form-label" for="login-email">{t("label-email")}</label>
                            <input
                                id="login-email"
                                class="form-control"
                                type="email"
                                placeholder="you@example.com"
                                value={(*email).clone()}
                                disabled={*loading}
                                oninput={on_email}
                                required=true
                            />
                        </div>
                        <div class="mb-3">
                            <label class="form-label" for="login-password">{t("label-password")}</label>
                            <input
                                id="login-password"
                                type="password"
                                class="form-control"
                                placeholder="Your password"
                                value={(*password).clone()}
                                disabled={*loading}
                                oninput={on_password}
                                required=true
                            />
                        </div>
                        <button class="btn btn-primary w-100" type="submit" disabled={*loading}>
                            if *loading {
                                <span class="spinner-border spinner-border-sm me-1"></span>
                            }
                            {if is_register { t("login-create-account-button") } else { t("login-sign-in") }}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
