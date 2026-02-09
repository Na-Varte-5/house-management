use crate::components::properties::{
    InvitationInfo, MeterCardList, MeterWithLastReading, PropertyHistoryTimeline, RenterInfo,
    RenterManagement, UserInfo,
};
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::i18n::{t, t_with_args};
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct ApartmentDetail {
    id: u64,
    number: String,
    building_id: u64,
    building_address: String,
    size_sq_m: Option<f64>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct ApartmentPermissions {
    can_view: bool,
    can_manage_renters: bool,
    can_view_meters: bool,
    is_owner: bool,
    is_renter: bool,
}

#[derive(Deserialize, Clone, PartialEq)]
struct RenterWithUser {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub user: UserInfo,
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Meters,
    Renters,
    History,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub apartment_id: u64,
}

#[function_component(PropertyDetailContent)]
pub fn property_detail_content(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let apartment = use_state(|| None::<ApartmentDetail>);
    let permissions = use_state(|| None::<ApartmentPermissions>);
    let meters = use_state(|| Vec::<MeterWithLastReading>::new());
    let renters = use_state(|| Vec::<RenterInfo>::new());
    let invitations = use_state(|| Vec::<InvitationInfo>::new());
    let all_users = use_state(|| Vec::<UserInfo>::new());
    let user_query = use_state(String::default);

    let active_tab = use_state(|| Tab::Meters);
    let loading = use_state(|| true);
    let loading_renters = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let apartment_id = props.apartment_id;
    let token = auth.token().map(|t| t.to_string());

    // Load apartment details and permissions on mount
    {
        let apartment = apartment.clone();
        let permissions = permissions.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(apartment_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());

                match client
                    .get::<ApartmentDetail>(&format!("/apartments/{}", id))
                    .await
                {
                    Ok(apt) => apartment.set(Some(apt)),
                    Err(e) => {
                        error.set(Some(t_with_args(
                            "properties-failed-load-apartment",
                            &[("error", &e.to_string())],
                        )));
                        loading.set(false);
                        return;
                    }
                }

                match client
                    .get::<ApartmentPermissions>(&format!("/apartments/{}/permissions", id))
                    .await
                {
                    Ok(perms) => permissions.set(Some(perms)),
                    Err(e) => {
                        error.set(Some(t_with_args(
                            "properties-failed-load-permissions",
                            &[("error", &e.to_string())],
                        )));
                    }
                }

                loading.set(false);
            });
            || ()
        });
    }

    // Load meters when tab selected
    {
        let meters = meters.clone();
        let error = error.clone();
        let token = token.clone();
        let active_tab_val = *active_tab;

        use_effect_with((apartment_id, active_tab_val), move |(id, tab)| {
            if matches!(tab, Tab::Meters) {
                let id = *id;
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<MeterWithLastReading>>(&format!("/apartments/{}/meters", id))
                        .await
                    {
                        Ok(list) => meters.set(list),
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-load-meters",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
            || ()
        });
    }

    // Load renters when tab selected
    {
        let renters = renters.clone();
        let invitations = invitations.clone();
        let loading_renters = loading_renters.clone();
        let error = error.clone();
        let token = token.clone();
        let active_tab_val = *active_tab;

        use_effect_with((apartment_id, active_tab_val), move |(id, tab)| {
            if matches!(tab, Tab::Renters) {
                let id = *id;
                loading_renters.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());

                    match client
                        .get::<Vec<RenterWithUser>>(&format!("/apartments/{}/renters", id))
                        .await
                    {
                        Ok(list) => {
                            let renter_infos: Vec<RenterInfo> = list
                                .into_iter()
                                .map(|r| RenterInfo {
                                    id: r.id,
                                    user_id: r.user_id,
                                    apartment_id: r.apartment_id,
                                    start_date: r.start_date,
                                    end_date: r.end_date,
                                    is_active: r.is_active,
                                    user_name: r.user.name,
                                    user_email: r.user.email,
                                })
                                .collect();
                            renters.set(renter_infos);
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-load-renters",
                            &[("error", &e.to_string())],
                        ))),
                    }

                    if let Ok(inv_list) = client
                        .get::<Vec<InvitationInfo>>(&format!("/apartments/{}/invitations", id))
                        .await
                    {
                        invitations.set(inv_list);
                    }

                    loading_renters.set(false);
                });
            }
            || ()
        });
    }

    // Load all users when renters tab is selected (for assignment dropdown)
    {
        let all_users = all_users.clone();
        let token = token.clone();
        let active_tab_val = *active_tab;

        use_effect_with(active_tab_val, move |tab| {
            if matches!(tab, Tab::Renters) {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(list) = client.get::<Vec<UserInfo>>("/users/public").await {
                        all_users.set(list);
                    }
                });
            }
            || ()
        });
    }

    let on_assign_renter = {
        let renters = renters.clone();
        let user_query = user_query.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(
            move |(user_id, start_date, end_date, is_active): (
                u64,
                Option<String>,
                Option<String>,
                bool,
            )| {
                let renters = renters.clone();
                let user_query = user_query.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = serde_json::json!({
                        "user_id": user_id,
                        "start_date": start_date,
                        "end_date": end_date,
                        "is_active": is_active,
                    });

                    match client
                        .post::<_, serde_json::Value>(
                            &format!("/apartments/{}/renters", apartment_id),
                            &payload,
                        )
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<RenterWithUser>>(&format!(
                                    "/apartments/{}/renters",
                                    apartment_id
                                ))
                                .await
                            {
                                let renter_infos: Vec<RenterInfo> = list
                                    .into_iter()
                                    .map(|r| RenterInfo {
                                        id: r.id,
                                        user_id: r.user_id,
                                        apartment_id: r.apartment_id,
                                        start_date: r.start_date,
                                        end_date: r.end_date,
                                        is_active: r.is_active,
                                        user_name: r.user.name,
                                        user_email: r.user.email,
                                    })
                                    .collect();
                                renters.set(renter_infos);
                                user_query.set(String::new());
                                success.set(Some(t("properties-renter-assigned")));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-assign",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            },
        )
    };

    let on_remove_renter = {
        let renters = renters.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            let renters = renters.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!(
                        "/apartments/{}/renters/{}",
                        apartment_id, user_id
                    ))
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client
                            .get::<Vec<RenterWithUser>>(&format!(
                                "/apartments/{}/renters",
                                apartment_id
                            ))
                            .await
                        {
                            let renter_infos: Vec<RenterInfo> = list
                                .into_iter()
                                .map(|r| RenterInfo {
                                    id: r.id,
                                    user_id: r.user_id,
                                    apartment_id: r.apartment_id,
                                    start_date: r.start_date,
                                    end_date: r.end_date,
                                    is_active: r.is_active,
                                    user_name: r.user.name,
                                    user_email: r.user.email,
                                })
                                .collect();
                            renters.set(renter_infos);
                            success.set(Some(t("properties-renter-removed")));
                        }
                    }
                    Err(e) => error.set(Some(t_with_args(
                        "properties-failed-remove",
                        &[("error", &e.to_string())],
                    ))),
                }
            });
        })
    };

    let on_toggle_renter_active = {
        let renters = renters.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();
        let renters_for_toggle = renters.clone();

        Callback::from(move |user_id: u64| {
            let current_renter = renters_for_toggle.iter().find(|r| r.user_id == user_id);
            if let Some(renter) = current_renter {
                let new_active_status = !renter.is_active;
                let renters = renters.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = serde_json::json!({
                        "is_active": new_active_status,
                    });

                    match client
                        .put::<_, serde_json::Value>(
                            &format!("/apartments/{}/renters/{}", apartment_id, user_id),
                            &payload,
                        )
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<RenterWithUser>>(&format!(
                                    "/apartments/{}/renters",
                                    apartment_id
                                ))
                                .await
                            {
                                let renter_infos: Vec<RenterInfo> = list
                                    .into_iter()
                                    .map(|r| RenterInfo {
                                        id: r.id,
                                        user_id: r.user_id,
                                        apartment_id: r.apartment_id,
                                        start_date: r.start_date,
                                        end_date: r.end_date,
                                        is_active: r.is_active,
                                        user_name: r.user.name,
                                        user_email: r.user.email,
                                    })
                                    .collect();
                                renters.set(renter_infos);
                                let status = if new_active_status {
                                    "active"
                                } else {
                                    "inactive"
                                };
                                success.set(Some(t_with_args(
                                    "properties-renter-marked",
                                    &[("status", status)],
                                )));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-update-renter",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
        })
    };

    let on_invite_renter = {
        let invitations = invitations.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(
            move |(email, start_date, end_date): (String, Option<String>, Option<String>)| {
                let invitations = invitations.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = serde_json::json!({
                        "email": email,
                        "start_date": start_date,
                        "end_date": end_date,
                    });

                    match client
                        .post::<_, serde_json::Value>(
                            &format!("/apartments/{}/invite", apartment_id),
                            &payload,
                        )
                        .await
                    {
                        Ok(response) => {
                            let status = response
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("sent");
                            let message = response
                                .get("message")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| t("properties-invitation-sent"));
                            success.set(Some(message));

                            if status == "pending" {
                                if let Ok(list) = client
                                    .get::<Vec<InvitationInfo>>(&format!(
                                        "/apartments/{}/invitations",
                                        apartment_id
                                    ))
                                    .await
                                {
                                    invitations.set(list);
                                }
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-invite",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            },
        )
    };

    let on_cancel_invitation = {
        let invitations = invitations.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |invitation_id: u64| {
            let invitations = invitations.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!(
                        "/apartments/{}/invitations/{}",
                        apartment_id, invitation_id
                    ))
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client
                            .get::<Vec<InvitationInfo>>(&format!(
                                "/apartments/{}/invitations",
                                apartment_id
                            ))
                            .await
                        {
                            invitations.set(list);
                        }
                        success.set(Some(t("properties-invitation-cancelled")));
                    }
                    Err(e) => error.set(Some(t_with_args(
                        "properties-failed-cancel-invitation",
                        &[("error", &e.to_string())],
                    ))),
                }
            });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::MyProperties))
    };

    let on_meter_click = {
        let navigator = navigator.clone();
        Callback::from(move |meter_id: u64| navigator.push(&Route::MeterDetail { id: meter_id }))
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    if *loading {
        return html! {
            <div class="container mt-4">
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            </div>
        };
    }

    if apartment.is_none() {
        return html! {
            <div class="container mt-4">
                <ErrorAlert
                    message={(*error).clone().unwrap_or_else(|| "Apartment not found".to_string())}
                    on_close={clear_error.clone()}
                />
                <button class="btn btn-outline-secondary mt-3" onclick={on_back.clone()}>
                    <i class="bi bi-arrow-left me-2"></i>{t("properties-back-to-properties")}
                </button>
            </div>
        };
    }

    let apt = apartment.as_ref().unwrap();
    let perms = permissions.as_ref();
    let can_manage_renters = perms.map(|p| p.can_manage_renters).unwrap_or(false);

    html! {
        <div class="container mt-4">
            <div class="d-flex justify-content-between align-items-start mb-4">
                <div>
                    <button class="btn btn-outline-secondary me-3" onclick={on_back}>
                        <i class="bi bi-arrow-left"></i>
                    </button>
                    <h2 class="d-inline mb-0">{t("properties-apartment-label")}{" "}{&apt.number}</h2>
                    <p class="text-muted mt-1 mb-0">
                        <i class="bi bi-building me-1"></i>
                        {&apt.building_address}
                        {
                            if let Some(size) = apt.size_sq_m {
                                html! {
                                    <span class="ms-3">
                                        <i class="bi bi-arrows-angle-expand me-1"></i>
                                        {format!("{:.1} m²", size)}
                                    </span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </p>
                </div>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }
            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            <ul class="nav nav-tabs mb-4">
                <li class="nav-item">
                    <button
                        class={if matches!(*active_tab, Tab::Meters) { "nav-link active" } else { "nav-link" }}
                        onclick={{
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| active_tab.set(Tab::Meters))
                        }}
                    >
                        <i class="bi bi-speedometer2 me-1"></i>{t("properties-meters-tab")}
                    </button>
                </li>
                if can_manage_renters {
                    <li class="nav-item">
                        <button
                            class={if matches!(*active_tab, Tab::Renters) { "nav-link active" } else { "nav-link" }}
                            onclick={{
                                let active_tab = active_tab.clone();
                                Callback::from(move |_| active_tab.set(Tab::Renters))
                            }}
                        >
                            <i class="bi bi-people me-1"></i>{t("properties-renters-tab")}
                        </button>
                    </li>
                }
                <li class="nav-item">
                    <button
                        class={if matches!(*active_tab, Tab::History) { "nav-link active" } else { "nav-link" }}
                        onclick={{
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| active_tab.set(Tab::History))
                        }}
                    >
                        <i class="bi bi-clock-history me-1"></i>{t("properties-history-tab")}
                    </button>
                </li>
            </ul>

            <div class="tab-content">
                {
                    match *active_tab {
                        Tab::Meters => html! {
                            <MeterCardList
                                meters={(*meters).clone()}
                                on_meter_click={on_meter_click}
                            />
                        },
                        Tab::Renters => html! {
                            <div class="card">
                                <div class="card-header">
                                    <h5 class="mb-0">{t("properties-manage-renters")}</h5>
                                </div>
                                <div class="card-body">
                                    <RenterManagement
                                        renters={(*renters).clone()}
                                        all_users={(*all_users).clone()}
                                        user_query={(*user_query).clone()}
                                        on_query_change={Callback::from(move |v: String| user_query.set(v))}
                                        on_assign={on_assign_renter}
                                        on_remove={on_remove_renter}
                                        on_toggle_active={on_toggle_renter_active}
                                        loading={*loading_renters}
                                        show={true}
                                        invitations={(*invitations).clone()}
                                        on_invite={Some(on_invite_renter.clone())}
                                        on_cancel_invitation={Some(on_cancel_invitation.clone())}
                                    />
                                </div>
                            </div>
                        },
                        Tab::History => html! {
                            <PropertyHistoryTimeline apartment_id={apartment_id} />
                        },
                    }
                }
            </div>
        </div>
    }
}
