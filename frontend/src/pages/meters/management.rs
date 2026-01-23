use yew::prelude::*;
use crate::components::{AdminLayout, ErrorAlert, SuccessAlert};
use crate::components::meters::{MeterRegisterForm, MeterList, MeterBuilding};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

#[derive(PartialEq, Clone)]
enum Tab {
    List,
    Register,
}

#[function_component(MeterManagementPage)]
pub fn meter_management_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    {"Access denied. Only Admins and Managers can access meter management."}
                </div>
            </div>
        };
    }

    let active_tab = use_state(|| Tab::List);
    let token = auth.token().map(|t| t.to_string());

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);
    let buildings = use_state(|| Vec::<MeterBuilding>::new());
    let reload_trigger = use_state(|| 0u32);

    // Load buildings for both tabs
    {
        let buildings = buildings.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<MeterBuilding>>("/buildings").await {
                    Ok(list) => buildings.set(list),
                    Err(e) => error.set(Some(format!("Failed to load buildings: {}", e))),
                }
            });
            || ()
        });
    }

    let on_tab_change = {
        let active_tab = active_tab.clone();
        let error = error.clone();
        let success = success.clone();
        move |tab: Tab| {
            active_tab.set(tab);
            error.set(None);
            success.set(None);
        }
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    let on_register_success = {
        let success = success.clone();
        let on_tab = on_tab_change.clone();
        let reload_trigger = reload_trigger.clone();
        Callback::from(move |_| {
            success.set(Some("Meter registered successfully!".to_string()));
            reload_trigger.set(*reload_trigger + 1);
            gloo_timers::callback::Timeout::new(2000, {
                let on_tab = on_tab.clone();
                move || {
                    on_tab(Tab::List);
                }
            }).forget();
        })
    };

    let on_error = {
        let error = error.clone();
        Callback::from(move |msg: String| error.set(Some(msg)))
    };

    html! {
        <AdminLayout title="Meter Management" active_route={Route::MeterManagement}>
            // Tabs
            <ul class="nav nav-tabs mb-3">
                <li class="nav-item">
                    <a
                        class={if matches!(*active_tab, Tab::List) { "nav-link active" } else { "nav-link" }}
                        style="cursor: pointer;"
                        onclick={let on_tab = on_tab_change.clone(); Callback::from(move |_| on_tab(Tab::List))}
                    >
                        {"List Meters"}
                    </a>
                </li>
                <li class="nav-item">
                    <a
                        class={if matches!(*active_tab, Tab::Register) { "nav-link active" } else { "nav-link" }}
                        style="cursor: pointer;"
                        onclick={let on_tab = on_tab_change.clone(); Callback::from(move |_| on_tab(Tab::Register))}
                    >
                        {"Register Meter"}
                    </a>
                </li>
            </ul>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if matches!(*active_tab, Tab::List) {
                <MeterList
                    buildings={(*buildings).clone()}
                    token={token.clone()}
                    reload_trigger={*reload_trigger}
                    on_error={on_error.clone()}
                />
            } else {
                <MeterRegisterForm
                    token={token.clone()}
                    on_success={on_register_success}
                    on_error={on_error}
                />
            }
        </AdminLayout>
    }
}
