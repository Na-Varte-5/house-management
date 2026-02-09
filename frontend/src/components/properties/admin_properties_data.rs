use crate::components::properties::*;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::i18n::{t, t_with_args};
use crate::services::api::{PaginatedResponse, api_client};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::contexts::AuthContext;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct RenterWithUser {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub user: UserInfo,
}

fn map_renters(list: Vec<RenterWithUser>) -> Vec<RenterInfo> {
    list.into_iter()
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
        .collect()
}

#[function_component(AdminPropertiesData)]
pub fn admin_properties_data() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| None::<u64>);
    let selected_apartment = use_state(|| None::<u64>);

    let address = use_state(String::default);
    let year = use_state(String::default);
    let apt_number = use_state(String::default);
    let apt_size = use_state(String::default);

    let apartment_owners = use_state(|| Vec::<UserInfo>::new());
    let apartment_renters = use_state(|| Vec::<RenterInfo>::new());
    let apartment_invitations = use_state(|| Vec::<InvitationInfo>::new());
    let all_users = use_state(|| Vec::<UserInfo>::new());
    let user_query = use_state(String::default);
    let renter_query = use_state(String::default);

    let loading_buildings = use_state(|| true);
    let loading_apartments = use_state(|| false);
    let loading_owners = use_state(|| false);
    let loading_renters = use_state(|| false);
    let submitting = use_state(|| false);

    let management_tab = use_state(|| "owners");

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    {
        let buildings = buildings.clone();
        let loading = loading_buildings.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<PaginatedResponse<Building>>("/buildings")
                    .await
                    .map(|r| r.data)
                {
                    Ok(list) => buildings.set(list),
                    Err(e) => error.set(Some(t_with_args(
                        "buildings-failed-load",
                        &[("error", &e.to_string())],
                    ))),
                }
                loading.set(false);
            });
            || ()
        });
    }

    {
        let apartments = apartments.clone();
        let selected = selected_building.clone();
        let loading = loading_apartments.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_building.clone(), move |_| {
            if let Some(bid) = *selected {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                        .await
                    {
                        Ok(list) => apartments.set(list),
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-load-apartments",
                            &[("error", &e.to_string())],
                        ))),
                    }
                    loading.set(false);
                });
            } else {
                apartments.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    {
        let owners = apartment_owners.clone();
        let selected = selected_apartment.clone();
        let loading = loading_owners.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_apartment.clone(), move |_| {
            if let Some(aid) = *selected {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                        .await
                    {
                        Ok(list) => owners.set(list),
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-load-owners",
                            &[("error", &e.to_string())],
                        ))),
                    }
                    loading.set(false);
                });
            } else {
                owners.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    {
        let renters = apartment_renters.clone();
        let invitations = apartment_invitations.clone();
        let selected = selected_apartment.clone();
        let loading = loading_renters.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_apartment.clone(), move |_| {
            if let Some(aid) = *selected {
                loading.set(true);
                let renters = renters.clone();
                let invitations = invitations.clone();
                let error = error.clone();
                let token = token.clone();
                let loading = loading.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<RenterWithUser>>(&format!("/apartments/{}/renters", aid))
                        .await
                    {
                        Ok(list) => renters.set(map_renters(list)),
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-load-renters",
                            &[("error", &e.to_string())],
                        ))),
                    }
                    if let Ok(inv_list) = client
                        .get::<Vec<InvitationInfo>>(&format!("/apartments/{}/invitations", aid))
                        .await
                    {
                        invitations.set(inv_list);
                    }
                    loading.set(false);
                });
            } else {
                renters.set(Vec::new());
                invitations.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    {
        let users = all_users.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<UserInfo>>("/users/public").await {
                    users.set(list);
                }
            });
            || ()
        });
    }

    let on_select_building = {
        let selected = selected_building.clone();
        let selected_apt = selected_apartment.clone();
        Callback::from(move |id: u64| {
            selected.set(Some(id));
            selected_apt.set(None);
        })
    };

    let on_select_apartment = {
        let selected = selected_apartment.clone();
        Callback::from(move |id: u64| selected.set(Some(id)))
    };

    let on_create_building = {
        let address = address.clone();
        let year = year.clone();
        let buildings = buildings.clone();
        let error = error.clone();
        let success = success.clone();
        let submitting = submitting.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let addr = (*address).clone();
            if addr.trim().is_empty() {
                error.set(Some(t("properties-address-required")));
                return;
            }

            let buildings = buildings.clone();
            let address = address.clone();
            let year = year.clone();
            let error = error.clone();
            let success = success.clone();
            let submitting = submitting.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);
            submitting.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_building = NewBuilding {
                    address: addr,
                    construction_year: year.parse::<i32>().ok(),
                };

                match client
                    .post::<_, serde_json::Value>("/buildings", &new_building)
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client
                            .get::<PaginatedResponse<Building>>("/buildings")
                            .await
                            .map(|r| r.data)
                        {
                            buildings.set(list);
                            address.set(String::new());
                            year.set(String::new());
                            success.set(Some(t("properties-building-created")));
                        }
                    }
                    Err(e) => error.set(Some(t_with_args(
                        "properties-failed-create-building",
                        &[("error", &e.to_string())],
                    ))),
                }
                submitting.set(false);
            });
        })
    };

    let on_create_apartment = {
        let apt_number = apt_number.clone();
        let apt_size = apt_size.clone();
        let selected_building = selected_building.clone();
        let apartments = apartments.clone();
        let error = error.clone();
        let success = success.clone();
        let submitting = submitting.clone();
        let token = token.clone();

        Callback::from(move |_| {
            if let Some(bid) = *selected_building {
                let num = (*apt_number).clone();
                if num.trim().is_empty() {
                    error.set(Some(t("properties-apt-number-required")));
                    return;
                }

                let apartments = apartments.clone();
                let apt_number = apt_number.clone();
                let apt_size = apt_size.clone();
                let error = error.clone();
                let success = success.clone();
                let submitting = submitting.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);
                submitting.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let new_apt = NewApartment {
                        building_id: bid,
                        number: num,
                        size_sq_m: apt_size.parse::<f64>().ok(),
                    };

                    match client
                        .post::<_, serde_json::Value>("/apartments", &new_apt)
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                                .await
                            {
                                apartments.set(list);
                                apt_number.set(String::new());
                                apt_size.set(String::new());
                                success.set(Some(t("properties-apt-created")));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-create-apartment",
                            &[("error", &e.to_string())],
                        ))),
                    }
                    submitting.set(false);
                });
            }
        })
    };

    let on_delete_building = {
        let buildings = buildings.clone();
        let selected_building = selected_building.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let buildings = buildings.clone();
            let selected_building = selected_building.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!("/buildings/{}", id))
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client
                            .get::<PaginatedResponse<Building>>("/buildings")
                            .await
                            .map(|r| r.data)
                        {
                            buildings.set(list);
                        }
                        if *selected_building == Some(id) {
                            selected_building.set(None);
                        }
                        success.set(Some(t("properties-building-deleted")));
                    }
                    Err(e) => error.set(Some(t_with_args(
                        "properties-failed-delete-building",
                        &[("error", &e.to_string())],
                    ))),
                }
            });
        })
    };

    let on_delete_apartment = {
        let apartments = apartments.clone();
        let selected_building = selected_building.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let apartments = apartments.clone();
            let selected_building = selected_building.clone();
            let selected_apartment = selected_apartment.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!("/apartments/{}", id))
                    .await
                {
                    Ok(_) => {
                        if let Some(bid) = *selected_building {
                            if let Ok(list) = client
                                .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                                .await
                            {
                                apartments.set(list);
                            }
                        }
                        if *selected_apartment == Some(id) {
                            selected_apartment.set(None);
                        }
                        success.set(Some(t("properties-apt-deleted")));
                    }
                    Err(e) => error.set(Some(t_with_args(
                        "properties-failed-delete-apartment",
                        &[("error", &e.to_string())],
                    ))),
                }
            });
        })
    };

    let on_assign_owner = {
        let apartment_owners = apartment_owners.clone();
        let selected_apartment = selected_apartment.clone();
        let user_query = user_query.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_owners = apartment_owners.clone();
                let user_query = user_query.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = AssignOwnerRequest { user_id };

                    match client
                        .post_no_response(&format!("/apartments/{}/owners", aid), &payload)
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                                .await
                            {
                                apartment_owners.set(list);
                                user_query.set(String::new());
                                success.set(Some(t("properties-owner-assigned")));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-assign-owner",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
        })
    };

    let on_remove_owner = {
        let apartment_owners = apartment_owners.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_owners = apartment_owners.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .delete_no_response(&format!("/apartments/{}/owners/{}", aid, user_id))
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                                .await
                            {
                                apartment_owners.set(list);
                                success.set(Some(t("properties-owner-removed")));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-remove-owner",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
        })
    };

    let on_assign_renter = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let renter_query = renter_query.clone();
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
                if let Some(aid) = *selected_apartment {
                    let apartment_renters = apartment_renters.clone();
                    let renter_query = renter_query.clone();
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
                                &format!("/apartments/{}/renters", aid),
                                &payload,
                            )
                            .await
                        {
                            Ok(_) => {
                                if let Ok(list) = client
                                    .get::<Vec<RenterWithUser>>(&format!(
                                        "/apartments/{}/renters",
                                        aid
                                    ))
                                    .await
                                {
                                    apartment_renters.set(map_renters(list));
                                    renter_query.set(String::new());
                                    success.set(Some(t("properties-renter-assigned")));
                                }
                            }
                            Err(e) => error.set(Some(t_with_args(
                                "properties-failed-assign",
                                &[("error", &e.to_string())],
                            ))),
                        }
                    });
                }
            },
        )
    };

    let on_remove_renter = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_renters = apartment_renters.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .delete_no_response(&format!("/apartments/{}/renters/{}", aid, user_id))
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<RenterWithUser>>(&format!("/apartments/{}/renters", aid))
                                .await
                            {
                                apartment_renters.set(map_renters(list));
                                success.set(Some(t("properties-renter-removed")));
                            }
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-remove",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
        })
    };

    let on_toggle_renter_active = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let current_renter = apartment_renters.iter().find(|r| r.user_id == user_id);
                if let Some(renter) = current_renter {
                    let new_active_status = !renter.is_active;
                    let apartment_renters = apartment_renters.clone();
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
                                &format!("/apartments/{}/renters/{}", aid, user_id),
                                &payload,
                            )
                            .await
                        {
                            Ok(_) => {
                                if let Ok(list) = client
                                    .get::<Vec<RenterWithUser>>(&format!(
                                        "/apartments/{}/renters",
                                        aid
                                    ))
                                    .await
                                {
                                    apartment_renters.set(map_renters(list));
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
            }
        })
    };

    let on_invite_renter = {
        let apartment_invitations = apartment_invitations.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(
            move |(email, start_date, end_date): (String, Option<String>, Option<String>)| {
                if let Some(aid) = *selected_apartment {
                    let apartment_invitations = apartment_invitations.clone();
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
                                &format!("/apartments/{}/invite", aid),
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
                                    .unwrap_or_else(|| t("properties-invitation-processed"));
                                success.set(Some(message));

                                if status == "pending" {
                                    if let Ok(list) = client
                                        .get::<Vec<InvitationInfo>>(&format!(
                                            "/apartments/{}/invitations",
                                            aid
                                        ))
                                        .await
                                    {
                                        apartment_invitations.set(list);
                                    }
                                }
                            }
                            Err(e) => error.set(Some(t_with_args(
                                "properties-failed-invite",
                                &[("error", &e.to_string())],
                            ))),
                        }
                    });
                }
            },
        )
    };

    let on_cancel_invitation = {
        let apartment_invitations = apartment_invitations.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |invitation_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_invitations = apartment_invitations.clone();
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
                            aid, invitation_id
                        ))
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<InvitationInfo>>(&format!(
                                    "/apartments/{}/invitations",
                                    aid
                                ))
                                .await
                            {
                                apartment_invitations.set(list);
                            }
                            success.set(Some(t("properties-invitation-cancelled")));
                        }
                        Err(e) => error.set(Some(t_with_args(
                            "properties-failed-cancel-invitation",
                            &[("error", &e.to_string())],
                        ))),
                    }
                });
            }
        })
    };

    html! {
        <div class="container-fluid">
            if let Some(msg) = (*error).clone() {
                <ErrorAlert message={msg} on_close={Callback::from(move |_| error.set(None))} />
            }
            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={Callback::from(move |_| success.set(None))} />
            }

            <div class="row">
                <div class="col-md-4">
                    <div class="card">
                        <div class="card-header">
                            <h5 class="mb-0">{t("properties-buildings")}</h5>
                        </div>
                        <div class="card-body">
                            <BuildingList
                                buildings={(*buildings).clone()}
                                selected_building_id={*selected_building}
                                on_select={on_select_building}
                                on_delete={on_delete_building}
                                loading={*loading_buildings}
                            />
                            <hr class="my-3" />
                            <BuildingForm
                                address={(*address).clone()}
                                year={(*year).clone()}
                                on_address_change={Callback::from(move |v| address.set(v))}
                                on_year_change={Callback::from(move |v| year.set(v))}
                                on_submit={on_create_building}
                                submitting={*submitting}
                            />
                        </div>
                    </div>
                </div>

                <div class="col-md-4">
                    <div class="card">
                        <div class="card-header">
                            <h5 class="mb-0">{t("properties-apartments")}</h5>
                        </div>
                        <div class="card-body">
                            <ApartmentList
                                apartments={(*apartments).clone()}
                                selected_apartment_id={*selected_apartment}
                                on_select={on_select_apartment}
                                on_delete={on_delete_apartment}
                                loading={*loading_apartments}
                                show={selected_building.is_some()}
                            />
                            <hr class="my-3" />
                            <ApartmentForm
                                number={(*apt_number).clone()}
                                size={(*apt_size).clone()}
                                on_number_change={Callback::from(move |v| apt_number.set(v))}
                                on_size_change={Callback::from(move |v| apt_size.set(v))}
                                on_submit={on_create_apartment}
                                submitting={*submitting}
                                show={selected_building.is_some()}
                            />
                        </div>
                    </div>
                </div>

                <div class="col-md-4">
                    <div class="card">
                        <div class="card-header">
                            <ul class="nav nav-tabs card-header-tabs" role="tablist">
                                <li class="nav-item" role="presentation">
                                    <button
                                        class={if *management_tab == "owners" { "nav-link active" } else { "nav-link" }}
                                        onclick={{
                                            let tab = management_tab.clone();
                                            Callback::from(move |_| tab.set("owners"))
                                        }}
                                        type="button"
                                    >
                                        {t("properties-owners-tab")}
                                    </button>
                                </li>
                                <li class="nav-item" role="presentation">
                                    <button
                                        class={if *management_tab == "renters" { "nav-link active" } else { "nav-link" }}
                                        onclick={{
                                            let tab = management_tab.clone();
                                            Callback::from(move |_| tab.set("renters"))
                                        }}
                                        type="button"
                                    >
                                        {t("properties-renters-tab-admin")}
                                    </button>
                                </li>
                                <li class="nav-item" role="presentation">
                                    <button
                                        class={if *management_tab == "history" { "nav-link active" } else { "nav-link" }}
                                        onclick={{
                                            let tab = management_tab.clone();
                                            Callback::from(move |_| tab.set("history"))
                                        }}
                                        type="button"
                                    >
                                        {t("properties-history-tab-admin")}
                                    </button>
                                </li>
                            </ul>
                        </div>
                        <div class="card-body">
                            if *management_tab == "owners" {
                                <OwnerManagement
                                    owners={(*apartment_owners).clone()}
                                    all_users={(*all_users).clone()}
                                    user_query={(*user_query).clone()}
                                    on_query_change={Callback::from(move |v| user_query.set(v))}
                                    on_assign={on_assign_owner}
                                    on_remove={on_remove_owner}
                                    loading={*loading_owners}
                                    show={selected_apartment.is_some()}
                                />
                            } else if *management_tab == "renters" {
                                <RenterManagement
                                    renters={(*apartment_renters).clone()}
                                    all_users={(*all_users).clone()}
                                    user_query={(*renter_query).clone()}
                                    on_query_change={Callback::from(move |v| renter_query.set(v))}
                                    on_assign={on_assign_renter}
                                    on_remove={on_remove_renter}
                                    on_toggle_active={on_toggle_renter_active}
                                    loading={*loading_renters}
                                    show={selected_apartment.is_some()}
                                    invitations={(*apartment_invitations).clone()}
                                    on_invite={Some(on_invite_renter.clone())}
                                    on_cancel_invitation={Some(on_cancel_invitation.clone())}
                                />
                            } else {
                                if let Some(apartment_id) = *selected_apartment {
                                    <PropertyHistoryTimeline apartment_id={apartment_id} />
                                } else {
                                    <div class="alert alert-info">
                                        <i class="bi bi-info-circle me-2"></i>
                                        {t("properties-select-apartment-history")}
                                    </div>
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
