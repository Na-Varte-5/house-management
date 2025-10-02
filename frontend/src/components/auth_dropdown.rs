use crate::utils::auth::set_token;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct LoginResponse {
    token: String,
}

#[function_component(AuthDropdown)]
pub fn auth_dropdown() -> Html {
    // prevent dropdown from closing on inside clicks (extra safety)
    let on_click_capture = Callback::from(|e: MouseEvent| e.stop_propagation());

    // State
    let email = use_state(String::default);
    let password = use_state(String::default);
    let name = use_state(String::default);
    let show_register = use_state(|| false);
    let message = use_state(|| None::<String>);

    // Submit handler (login or register based on toggle)
    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let name = name.clone();
        let show_register = show_register.clone();
        let message = message.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let email_v = (*email).clone();
            let pass_v = (*password).clone();
            let registering = *show_register;
            let name_v = (*name).clone();
            let message = message.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if registering {
                    let payload = serde_json::json!({
                        "email": email_v,
                        "name": name_v,
                        "password": pass_v,
                    });
                    match reqwasm::http::Request::post("/api/v1/auth/register")
                        .header("Content-Type", "application/json")
                        .body(payload.to_string())
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            if resp.ok() || resp.status() == 201 {
                                // Auto-login
                                let login_payload = serde_json::json!({
                                    "email": payload["email"].as_str().unwrap_or_default(),
                                    "password": payload["password"].as_str().unwrap_or_default(),
                                });
                                if let Ok(login_resp) =
                                    reqwasm::http::Request::post("/api/v1/auth/login")
                                        .header("Content-Type", "application/json")
                                        .body(login_payload.to_string())
                                        .send()
                                        .await
                                {
                                    if login_resp.ok() {
                                        if let Ok(LoginResponse { token }) =
                                            login_resp.json::<LoginResponse>().await
                                        {
                                            set_token(&token);
                                            if let Some(w) = web_sys::window() {
                                                let _ = w.location().reload();
                                            }
                                        } else {
                                            message.set(Some(
                                                "Registered, but failed to parse login response"
                                                    .into(),
                                            ));
                                        }
                                    } else {
                                        message.set(Some("Registered. Please login.".into()));
                                    }
                                }
                            } else {
                                message.set(Some("Registration failed".into()));
                            }
                        }
                        Err(_) => message.set(Some("Network error".into())),
                    }
                } else {
                    let payload = serde_json::json!({
                        "email": email_v,
                        "password": pass_v,
                    });
                    match reqwasm::http::Request::post("/api/v1/auth/login")
                        .header("Content-Type", "application/json")
                        .body(payload.to_string())
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(LoginResponse { token }) =
                                    resp.json::<LoginResponse>().await
                                {
                                    set_token(&token);
                                    if let Some(w) = web_sys::window() {
                                        let _ = w.location().reload();
                                    }
                                } else {
                                    message.set(Some("Invalid response".into()));
                                }
                            } else {
                                message.set(Some("Login failed".into()));
                            }
                        }
                        Err(_) => message.set(Some("Network error".into())),
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
            if let Some(msg) = (*message).clone() { <div class="alert alert-warning py-1 px-2 mb-2">{msg}</div> }
            if *show_register {
                <input class="form-control form-control-sm" placeholder="Full name" value={(*name).clone()}
                    oninput={{ let s=name.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(input.value()); }) }} />
            }
            <input class="form-control form-control-sm" placeholder="Email" value={(*email).clone()}
                oninput={{ let s=email.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(input.value()); }) }} />
            <input type="password" class="form-control form-control-sm" placeholder="Password" value={(*password).clone()}
                oninput={{ let s=password.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(input.value()); }) }} />
            <div class="d-flex justify-content-between align-items-center mt-1">
                <button class="btn btn-sm btn-primary" type="submit">{ if *show_register { "Register" } else { "Login" } }</button>
                <button class="btn btn-sm btn-link text-decoration-none" type="button" onclick={toggle_register}>
                    { if *show_register { "Have an account? Login" } else { "Create account" } }
                </button>
            </div>
        </form>
    }
}
