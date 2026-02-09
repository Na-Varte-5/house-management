use crate::components::ErrorAlert;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::services::{ApiError, api_client};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct UserWithRoles {
    id: u64,
    email: String,
    name: String,
    roles: Vec<String>,
}

#[derive(Serialize)]
struct UpdateRolesRequest {
    roles: Vec<String>,
}

const ALL_ROLES: &[&str] = &["Admin", "Manager", "Homeowner", "Renter", "HOA Member"];

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let users = use_state(|| Vec::<UserWithRoles>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Only admins can access
    if !auth.has_role("Admin") {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    <strong>{t("admin-access-denied")}</strong>
                    <p class="mb-0 small">{"Only Admins can access user management."}</p>
                </div>
            </div>
        };
    }

    // Load users on mount
    {
        let users = users.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<UserWithRoles>>("/users/with_roles").await {
                    Ok(list) => {
                        users.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load users: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let reload_users = {
        let users = users.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |_| {
            let users = users.clone();
            let error = error.clone();
            let token = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<UserWithRoles>>("/users/with_roles").await {
                    Ok(list) => users.set(list),
                    Err(e) => error.set(Some(format!("Failed to reload users: {}", e))),
                }
            });
        })
    };

    let on_add_role = {
        let error = error.clone();
        let success = success.clone();
        let reload_users = reload_users.clone();
        let users_state = users.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |(user_id, role): (u64, String)| {
            let error = error.clone();
            let success = success.clone();
            let reload_users = reload_users.clone();
            let users_state = users_state.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                // Find current user and add role
                let current = users_state.iter().find(|u| u.id == user_id).cloned();
                if let Some(mut user_record) = current {
                    if !user_record.roles.contains(&role) {
                        user_record.roles.push(role.clone());
                    }

                    let client = api_client(token.as_deref());
                    let update_req = UpdateRolesRequest {
                        roles: user_record.roles,
                    };

                    match client
                        .post::<_, serde_json::Value>(
                            &format!("/users/{}/roles", user_id),
                            &update_req,
                        )
                        .await
                    {
                        Ok(_) => {
                            success.set(Some(format!("Role '{}' added successfully", role)));
                            reload_users.emit(());
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some(
                                "You don't have permission to update roles".to_string(),
                            ));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to add role: {}", e)));
                        }
                    }
                }
            });
        })
    };

    let on_remove_role = {
        let error = error.clone();
        let success = success.clone();
        let reload_users = reload_users.clone();
        let users_state = users.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |(user_id, role): (u64, String)| {
            let error = error.clone();
            let success = success.clone();
            let reload_users = reload_users.clone();
            let users_state = users_state.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                // Find current user and remove role
                let current = users_state.iter().find(|u| u.id == user_id).cloned();
                if let Some(mut user_record) = current {
                    user_record.roles.retain(|r| r != &role);

                    let client = api_client(token.as_deref());
                    let update_req = UpdateRolesRequest {
                        roles: user_record.roles,
                    };

                    match client
                        .post::<_, serde_json::Value>(
                            &format!("/users/{}/roles", user_id),
                            &update_req,
                        )
                        .await
                    {
                        Ok(_) => {
                            success.set(Some(format!("Role '{}' removed successfully", role)));
                            reload_users.emit(());
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some(
                                "You don't have permission to update roles".to_string(),
                            ));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to remove role: {}", e)));
                        }
                    }
                }
            });
        })
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    html! {
        <>
            <h2 class="mb-3">{"User Management"}</h2>
            <div class="card">
                <div class="card-body">
                    if let Some(err) = (*error).clone() {
                        <ErrorAlert message={err} on_close={clear_error.clone()} />
                    }

                    if let Some(msg) = (*success).clone() {
                        <div class="alert alert-success alert-dismissible fade show" role="alert">
                            {msg}
                            <button type="button" class="btn-close" onclick={clear_success.clone()}></button>
                        </div>
                    }

                    if *loading {
                        <div class="text-center py-5">
                            <div class="spinner-border" role="status">
                                <span class="visually-hidden">{t("user-mgmt-loading")}</span>
                            </div>
                        </div>
                    } else if (*users).is_empty() {
                        <div class="alert alert-info">{t("user-mgmt-no-users")}</div>
                    } else {
                        <div class="table-responsive">
                            <table class="table table-sm table-striped mb-0">
                                <thead>
                                    <tr>
                                        <th>{t("user-mgmt-id")}</th>
                                        <th>{t("user-mgmt-name")}</th>
                                        <th>{t("user-mgmt-email")}</th>
                                        <th>{t("user-mgmt-roles")}</th>
                                        <th>{t("user-mgmt-add-role")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { for users.iter().map(|u| {
                                        let uid = u.id;
                                        html! {
                                            <tr>
                                                <td>{u.id}</td>
                                                <td>{&u.name}</td>
                                                <td class="small">{&u.email}</td>
                                                <td>
                                                    { for u.roles.iter().map(|r| {
                                                        let role = r.clone();
                                                        let cb = on_remove_role.clone();
                                                        html! {
                                                            <span class="badge bg-secondary me-1">
                                                                {&role}
                                                                {" "}
                                                                <button
                                                                    type="button"
                                                                    class="btn-close btn-close-white"
                                                                    style="font-size: 0.5rem; vertical-align: middle;"
                                                                    aria-label="Remove role"
                                                                    onclick={Callback::from(move |_| cb.emit((uid, role.clone())))}
                                                                ></button>
                                                            </span>
                                                        }
                                                    }) }
                                                </td>
                                                <td>
                                                    <select
                                                        class="form-select form-select-sm"
                                                        onchange={{
                                                            let cb = on_add_role.clone();
                                                            Callback::from(move |e: Event| {
                                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                                let val = select.value();
                                                                if !val.is_empty() {
                                                                    cb.emit((uid, val.clone()));
                                                                    select.set_value("");  // Reset select
                                                                }
                                                            })
                                                        }}
                                                    >
                                                        <option value="">{t("user-mgmt-select-role")}</option>
                                                        { for ALL_ROLES.iter().map(|r| {
                                                            html! { <option value={r.to_string()}>{r.to_string()}</option> }
                                                        }) }
                                                    </select>
                                                </td>
                                            </tr>
                                        }
                                    }) }
                                </tbody>
                            </table>
                        </div>
                        <p class="small text-muted mt-3 mb-0">
                            {t("user-mgmt-help")}
                        </p>
                    }
                </div>
            </div>
        </>
    }
}
