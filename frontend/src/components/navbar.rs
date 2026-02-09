use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::language_switcher::LanguageSwitcher;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::routes::Route;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let on_logout = {
        let auth = auth.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            auth.logout.emit(());
            navigator.push(&Route::Home);
        })
    };

    html! {
        <nav class="navbar navbar-expand navbar-dark bg-dark fixed-top">
            <div class="container-fluid">
                if auth.is_authenticated() {
                    <button class="btn btn-sm btn-outline-light me-2 sidebar-mobile-toggle"
                            type="button" data-bs-toggle="offcanvas" data-bs-target="#mobileSidebar">
                        <i class="bi bi-list"></i>
                    </button>
                }
                <Link<Route> to={Route::Home} classes="navbar-brand">{ t("app-name") }</Link<Route>>

                <div class="d-flex ms-auto">
                    <div class="me-2">
                        <LanguageSwitcher />
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
                        <Link<Route> to={Route::Login} classes="btn btn-sm btn-outline-light">
                            { t("nav-login") }
                        </Link<Route>>
                    }
                </div>
            </div>
        </nav>
    }
}
