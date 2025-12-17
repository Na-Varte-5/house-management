use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::auth_dropdown::AuthDropdown;
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::i18n::{set_language, current_language, available_languages, t};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let on_logout = {
        let auth = auth.clone();
        Callback::from(move |_| {
            auth.logout.emit(());
            if let Some(w) = web_sys::window() {
                let _ = w.location().reload();
            }
        })
    };

    let lang_state = use_state(|| current_language());
    let on_lang_change = {
        let lang_state = lang_state.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let val = select.value();
            set_language(&val);
            lang_state.set(val);
        })
    };

    let is_manager_or_admin = auth.is_admin_or_manager();

    html! {
        <nav class="navbar navbar-expand navbar-dark bg-dark">
            <div class="container-fluid">
                <Link<Route> to={Route::Home} classes="navbar-brand">{ t("app-name") }</Link<Route>>
                <div class="navbar-nav">
                    <Link<Route> to={Route::Buildings} classes="nav-link">{ t("nav-buildings") }</Link<Route>>
                    { if auth.is_authenticated() { html!{
                        <>
                            <Link<Route> to={Route::Maintenance} classes="nav-link">{"Maintenance"}</Link<Route>>
                            <Link<Route> to={Route::Voting} classes="nav-link">{"Voting"}</Link<Route>>
                        </>
                    } } else { html!{} } }
                    <Link<Route> to={Route::Health} classes="nav-link">{ t("nav-health") }</Link<Route>>
                    { if is_manager_or_admin { html!{
                        <Link<Route> to={Route::Manage} classes="nav-link">{ t("nav-dashboard") }</Link<Route>>
                    } } else { html!{} } }
                </div>
                <div class="d-flex">
                    <div class="me-2">
                        <select class="form-select form-select-sm bg-dark text-light border-secondary" onchange={on_lang_change.clone()} value={(*lang_state).clone()}>
                            { for available_languages().into_iter().map(|code| html!{<option value={code.clone()} selected={code==*lang_state}>{code.to_uppercase()}</option>}) }
                        </select>
                    </div>
                    if auth.is_authenticated() {
                        if let Some(u) = auth.user() {
                            <div class="dropdown" data-bs-auto-close="outside">
                                <button class="btn btn-sm btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">
                                    {u.name.clone()}
                                    {
                                        if u.roles.iter().any(|r| r=="Admin" || r=="Manager") {
                                            html!{<span class="badge bg-warning text-dark ms-1">{"★"}</span>}
                                        } else {
                                            html!{}
                                        }
                                    }
                                </button>
                                <div class="dropdown-menu dropdown-menu-end p-2" style="min-width: 260px;">
                                    <div class="mb-2">
                                        <div class="fw-semibold">{u.name.clone()}</div>
                                        <div class="small text-muted">{u.email.clone()}</div>
                                    </div>
                                    <div class="mb-2 d-flex flex-wrap gap-1">
                                        { for u.roles.iter().map(|r| {
                                            let key = match r.as_str() {"Admin"=>"role-admin","Manager"=>"role-manager","Homeowner"=>"role-homeowner","Renter"=>"role-renter","HOA Member"=>"role-hoa-member", _=>""};
                                            html!{<span class="badge bg-secondary">{ if key.is_empty() { r.clone() } else { t(key) } }</span>}
                                        }) }
                                    </div>
                                    <button class="btn btn-sm btn-outline-danger w-100" onclick={on_logout}> { t("nav-logout") } </button>
                                </div>
                            </div>
                        } else {
                            <button class="btn btn-sm btn-outline-light" onclick={on_logout}>{ t("nav-logout") }</button>
                        }
                    } else {
                        <div class="dropdown ms-2" data-bs-auto-close="outside">
                            <button class="btn btn-sm btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">{ t("nav-login") }</button>
                            <div class="dropdown-menu dropdown-menu-end p-3" style="min-width: 280px;">
                                <AuthDropdown />
                            </div>
                        </div>
                    }
                </div>
            </div>
        </nav>
    }
}
