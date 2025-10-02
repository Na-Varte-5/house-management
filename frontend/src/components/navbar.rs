use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::auth_dropdown::AuthDropdown;
use crate::routes::Route;
use crate::utils::auth::{clear_token, get_token};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let logged_in = use_state(|| get_token().is_some());
    {
        let logged_in = logged_in.clone();
        use_effect_with((), move |_| {
            logged_in.set(get_token().is_some());
            || ()
        });
    }

    let on_logout = {
        let logged_in = logged_in.clone();
        Callback::from(move |_| {
            clear_token();
            logged_in.set(false);
            if let Some(w) = web_sys::window() {
                let _ = w.location().reload();
            }
        })
    };

    html! {
        <nav class="navbar navbar-expand navbar-dark bg-dark">
            <div class="container-fluid">
                <Link<Route> to={Route::Home} classes="navbar-brand">{"HouseMgmt"}</Link<Route>>
                <div class="navbar-nav">
                    <Link<Route> to={Route::Buildings} classes="nav-link">{"Buildings"}</Link<Route>>
                </div>
                <div class="d-flex">
                    if *logged_in {
                        <button class="btn btn-sm btn-outline-light ms-2" onclick={on_logout}>{"Logout"}</button>
                    } else {
                        <div class="dropdown ms-2" data-bs-auto-close="outside">
                            <button class="btn btn-sm btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">{"Login"}</button>
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
