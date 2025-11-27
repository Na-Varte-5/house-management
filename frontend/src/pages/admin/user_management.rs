// ...existing AdminPage implementation moved from pages/admin.rs...
use crate::components::AdminLayout;
use yew::prelude::*;
use crate::utils::auth::{current_user, get_token};
use crate::utils::api::api_url;

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
struct UserWithRoles { id: u64, email: String, name: String, roles: Vec<String> }

const ALL_ROLES: &[&str] = &["Admin", "Manager", "Homeowner", "Renter", "HOAMember"];

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    // ...existing code from pages/admin.rs...
    let user = current_user();
    let users = use_state(|| Vec::<UserWithRoles>::new());
    let message = use_state(|| None::<String>);

    let is_admin = user.as_ref().map(|u| u.roles.iter().any(|r| r == "Admin")).unwrap_or(false);
    {
        let users = users.clone();
        let message = message.clone();
        use_effect_with(is_admin, move |_| {
            if is_admin {
                let users = users.clone();
                let message = message.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::get(&api_url("/api/v1/users/with_roles"));
                    if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(list) = resp.json::<Vec<UserWithRoles>>().await { users.set(list); }
                            } else { message.set(Some("Failed to load users".into())); }
                        }
                        Err(_) => message.set(Some("Network error".into())),
                    }
                });
            }
            || ()
        });
    }

    let on_add_role = {
        let users = users.clone();
        let message = message.clone();
        Callback::from(move |(user_id, role): (u64, String)| {
            let users = users.clone();
            let message = message.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let current = users.iter().find(|u| u.id == user_id).cloned();
                if let Some(mut urec) = current {
                    if !urec.roles.contains(&role) { urec.roles.push(role.clone()); }
                    let payload = serde_json::json!({"roles": urec.roles});
                    let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/users/{}/roles", user_id)))
                        .header("Content-Type", "application/json");
                    if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.body(payload.to_string()).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                // reload list
                                let mut req2 = reqwasm::http::Request::get(&api_url("/api/v1/users/with_roles"));
                                if let Some(tok) = get_token() { req2 = req2.header("Authorization", &format!("Bearer {}", tok)); }
                                if let Ok(resp2) = req2.send().await { if let Ok(list) = resp2.json::<Vec<UserWithRoles>>().await { users.set(list); } }
                            } else { message.set(Some("Update failed".into())); }
                        }
                        Err(_) => message.set(Some("Network error".into())),
                    }
                }
            });
        })
    };

    let on_remove_role = {
        let users = users.clone();
        let message = message.clone();
        Callback::from(move |(user_id, role): (u64, String)| {
            let users = users.clone();
            let message = message.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let current = users.iter().find(|u| u.id == user_id).cloned();
                if let Some(mut urec) = current {
                    urec.roles.retain(|r| r != &role);
                    let payload = serde_json::json!({"roles": urec.roles});
                    let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/users/{}/roles", user_id)))
                        .header("Content-Type", "application/json");
                    if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.body(payload.to_string()).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                let mut req2 = reqwasm::http::Request::get(&api_url("/api/v1/users/with_roles"));
                                if let Some(tok) = get_token() { req2 = req2.header("Authorization", &format!("Bearer {}", tok)); }
                                if let Ok(resp2) = req2.send().await { if let Ok(list) = resp2.json::<Vec<UserWithRoles>>().await { users.set(list); } }
                            } else { message.set(Some("Update failed".into())); }
                        }
                        Err(_) => message.set(Some("Network error".into())),
                    }
                }
            });
        })
    };

    if !is_admin {
        return html!{<div class="container mt-4"><div class="alert alert-danger">{"Access denied: Admins only."}</div></div>};
    }

    html! {
        <AdminLayout title={"Admin - User Management".to_string()} active_route={crate::routes::Route::Admin}>
            <div class="card">
                <div class="card-body">
                    if let Some(msg) = (*message).clone() { <div class="alert alert-warning py-1 px-2">{msg}</div> }
                    <table class="table table-sm table-striped mb-0">
                        <thead><tr><th>{"ID"}</th><th>{"Name"}</th><th>{"Email"}</th><th>{"Roles"}</th><th>{"Add Role"}</th></tr></thead>
                        <tbody>
                            { for users.iter().map(|u| {
                                let uid = u.id;
                                html!{<tr>
                                    <td>{u.id}</td>
                                    <td>{u.name.clone()}</td>
                                    <td class="small">{u.email.clone()}</td>
                                    <td>
                                        { for u.roles.iter().map(|r| {
                                            let role = r.clone();
                                            let cb = on_remove_role.clone();
                                            html!{<span class="badge bg-secondary me-1">{role.clone()} <button type="button" class="btn-close btn-close-white btn-sm ms-1" style="font-size:0.5rem" onclick={Callback::from(move |_| cb.emit((uid, role.clone())))}></button></span>}
                                        }) }
                                    </td>
                                    <td>
                                        <select class="form-select form-select-sm" onchange={{
                                            let cb = on_add_role.clone();
                                            Callback::from(move |e: Event| {
                                                let sel: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                let val = sel.value();
                                                if !val.is_empty() { cb.emit((uid, val)); }
                                            })
                                        }}>
                                            <option value="">{"Select"}</option>
                                            { for ALL_ROLES.iter().map(|r| html!{<option value={r.to_string()}>{r.to_string()}</option>}) }
                                        </select>
                                    </td>
                                </tr>}
                            }) }
                        </tbody>
                    </table>
                    <p class="small text-muted mt-2">{"Remove a role by clicking the × on its badge."}</p>
                </div>
            </div>
        </AdminLayout>
    }
}
