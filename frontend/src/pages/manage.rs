use yew::prelude::*;
use crate::utils::auth::current_user;
use crate::utils::api::api_url;
use crate::components::spinner::Spinner;
use serde::Deserialize;
use crate::components::announcements::AnnouncementsManage;

#[derive(Deserialize, Clone, PartialEq)]
struct Building { id: u64, address: String, construction_year: Option<i32> }
#[derive(Deserialize, Clone, PartialEq)]
struct Apartment { id: u64, building_id: u64, number: String, size_sq_m: Option<f64> }

#[function_component(ManagePage)]
pub fn manage_page() -> Html {
    let user = current_user();
    let can_manage = user.as_ref().map(|u| u.roles.iter().any(|r| r=="Admin" || r=="Manager")).unwrap_or(false);
    if !can_manage { return html!{<div class="container mt-4"><div class="alert alert-danger">{"Access denied"}</div></div>}; }

    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| None::<u64>);
    let message = use_state(|| None::<String>);

    let address = use_state(String::default);
    let year = use_state(String::default);
    let apt_number = use_state(String::default);
    let apt_size = use_state(String::default);
    let pending_delete_building = use_state(|| None::<u64>);
    let pending_delete_apartment = use_state(|| None::<u64>);

    let selected_apartment = use_state(|| None::<u64>);
    let apartment_owners = use_state(|| Vec::<(u64,String,String)>::new()); // (id,name,email)
    let all_users = use_state(|| Vec::<(u64,String,String)>::new());
    let user_query = use_state(String::default);

    let deleted_buildings = use_state(|| Vec::<Building>::new());
    let deleted_apartments = use_state(|| Vec::<Apartment>::new());
    let show_deleted = use_state(|| false);

    let loading_buildings = use_state(|| false);
    let loading_apartments = use_state(|| false);
    let loading_owners = use_state(|| false);
    let loading_deleted = use_state(|| false);

    // load buildings
    {
        let buildings = buildings.clone();
        let loading = loading_buildings.clone();
        use_effect_with((), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = reqwasm::http::Request::get(&api_url("/api/v1/buildings")).send().await {
                    if let Ok(list) = resp.json::<Vec<Building>>().await { buildings.set(list); }
                }
                loading.set(false);
            });
            || ()
        });
    }
    // load apartments when building selected
    {
        let apartments = apartments.clone();
        let sel = selected_building.clone();
        let loading_ap = loading_apartments.clone();
        use_effect_with(selected_building.clone(), move |_| {
            if let Some(bid) = *sel {
                loading_ap.set(true);
                let apartments2 = apartments.clone();
                let loading_ap2 = loading_ap.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/v1/buildings/{}/apartments", bid);
                    if let Ok(resp) = reqwasm::http::Request::get(&api_url(&url)).send().await {
                        if let Ok(list) = resp.json::<Vec<Apartment>>().await { apartments2.set(list); }
                    }
                    loading_ap2.set(false);
                });
            } else { apartments.set(Vec::new()); loading_ap.set(false); }
            || ()
        });
    }
    // load owners when selected_apartment changes
    {
        let owners_state = apartment_owners.clone();
        let sel_ap = selected_apartment.clone();
        let loading_owners_state = loading_owners.clone();
        use_effect_with(selected_apartment.clone(), move |_| {
            if let Some(aid) = *sel_ap {
                loading_owners_state.set(true);
                let owners_state2 = owners_state.clone();
                let loading_owners_state2 = loading_owners_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/v1/apartments/{}/owners", aid);
                    if let Ok(resp) = reqwasm::http::Request::get(&api_url(&url)).send().await {
                        if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                            let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("name")?.as_str()?.to_string(), v.get("email")?.as_str()?.to_string()))).collect();
                            owners_state2.set(mapped);
                        }
                    }
                    loading_owners_state2.set(false);
                });
            } else { owners_state.set(Vec::new()); loading_owners_state.set(false); }
            || ()
        });
    }
    // load public users once (for search) if can_manage
    {
        let can = can_manage;
        let all_users_state = all_users.clone();
        use_effect_with(can, move |_| {
            if can {
                let all_users_state2 = all_users_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::get(&api_url("/api/v1/users/public"));
                    if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    if let Ok(resp) = req.send().await { if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                        let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("name")?.as_str()?.to_string(), v.get("email")?.as_str()?.to_string()))).collect();
                        all_users_state2.set(mapped);
                    }}
                });
            }
            || ()
        });
    }

    // add building handler
    let on_add_building = {
        let address = address.clone();
        let year = year.clone();
        let buildings_state = buildings.clone();
        let message_state = message.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let addr = (*address).clone();
            if addr.trim().is_empty() { message_state.set(Some("Address required".into())); return; }
            let yr = year.parse::<i32>().ok();
            let payload = serde_json::json!({"address": addr, "construction_year": yr});
            let buildings_state2 = buildings_state.clone();
            let address2 = address.clone();
            let year2 = year.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url("/api/v1/buildings")).header("Content-Type","application/json");
                if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                if let Ok(resp) = req.body(payload.to_string()).send().await { if resp.ok() {
                    if let Ok(r2) = reqwasm::http::Request::get(&api_url("/api/v1/buildings")).send().await { if let Ok(list) = r2.json::<Vec<Building>>().await { buildings_state2.set(list); address2.set(String::new()); year2.set(String::new()); } }
                }}
            });
        })
    };

    // add apartment handler
    let on_add_apartment = {
        let sel = selected_building.clone();
        let apt_number = apt_number.clone();
        let apt_size = apt_size.clone();
        let apartments_state = apartments.clone();
        let message_state = message.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(bid) = *sel {
                let num = (*apt_number).clone();
                if num.trim().is_empty() { message_state.set(Some("Number required".into())); return; }
                let size = apt_size.parse::<f64>().ok();
                let payload = serde_json::json!({"building_id": bid, "number": num, "size_sq_m": size});
                let apartments2 = apartments_state.clone();
                let apt_number2 = apt_number.clone();
                let apt_size2 = apt_size.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::post(&api_url("/api/v1/apartments")).header("Content-Type","application/json");
                    if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    if let Ok(resp) = req.body(payload.to_string()).send().await { if resp.ok() {
                        let url = format!("/api/v1/buildings/{}/apartments", bid);
                        if let Ok(r2) = reqwasm::http::Request::get(&api_url(&url)).send().await { if let Ok(list) = r2.json::<Vec<Apartment>>().await { apartments2.set(list); apt_number2.set(String::new()); apt_size2.set(String::new()); } }
                    }}
                });
            }
        })
    };

    // soft delete handlers
    let on_delete_building = {
        let buildings_state = buildings.clone();
        let selected_building_state = selected_building.clone();
        let show_deleted_state = show_deleted.clone();
        let deleted_buildings_state = deleted_buildings.clone();
        Callback::from(move |id: u64| {
            let buildings_state2 = buildings_state.clone();
            let selected_building_state2 = selected_building_state.clone();
            let show_deleted_state2 = show_deleted_state.clone();
            let deleted_buildings_state2 = deleted_buildings_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::delete(&api_url(&format!("/api/v1/buildings/{}", id)));
                if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                if let Ok(resp) = req.send().await { if resp.ok() {
                    // reload active buildings
                    if let Ok(r2) = reqwasm::http::Request::get(&api_url("/api/v1/buildings")).send().await { if let Ok(list) = r2.json::<Vec<Building>>().await { buildings_state2.set(list); } }
                    // reload deleted if visible
                    if *show_deleted_state2 { if let Ok(r3) = reqwasm::http::Request::get(&api_url("/api/v1/buildings/deleted")).send().await { if let Ok(list) = r3.json::<Vec<Building>>().await { deleted_buildings_state2.set(list); } } }
                    if *selected_building_state2 == Some(id) { selected_building_state2.set(None); }
                }}
            });
        })
    };
    let on_delete_apartment = {
        let apartments_state = apartments.clone();
        let sel_b = selected_building.clone();
        let show_deleted_state = show_deleted.clone();
        let deleted_apartments_state = deleted_apartments.clone();
        Callback::from(move |id: u64| {
            let apartments_state2 = apartments_state.clone();
            let sel_b2 = sel_b.clone();
            let show_deleted_state2 = show_deleted_state.clone();
            let deleted_apartments_state2 = deleted_apartments_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::delete(&api_url(&format!("/api/v1/apartments/{}", id)));
                if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                if let Ok(resp) = req.send().await { if resp.ok() {
                    if let Some(bid) = *sel_b2 { let url = format!("/api/v1/buildings/{}/apartments", bid); if let Ok(r2) = reqwasm::http::Request::get(&api_url(&url)).send().await { if let Ok(list) = r2.json::<Vec<Apartment>>().await { apartments_state2.set(list); } } }
                    if *show_deleted_state2 { if let Ok(r3) = reqwasm::http::Request::get(&api_url("/api/v1/apartments/deleted")).send().await { if let Ok(list) = r3.json::<Vec<Apartment>>().await { deleted_apartments_state2.set(list); } } }
                }}
            });
        })
    };

    let confirm_delete_building = {
        let pending = pending_delete_building.clone();
        let del = on_delete_building.clone();
        Callback::from(move |_: MouseEvent| { if let Some(id) = *pending { del.emit(id); } pending.set(None); })
    };
    let confirm_delete_apartment = {
        let pending = pending_delete_apartment.clone();
        let del = on_delete_apartment.clone();
        Callback::from(move |_: MouseEvent| { if let Some(id) = *pending { del.emit(id); } pending.set(None); })
    };
    let cancel_delete = {
        let p1 = pending_delete_building.clone();
        let p2 = pending_delete_apartment.clone();
        Callback::from(move |_: MouseEvent| { p1.set(None); p2.set(None); })
    };

    // owner assignment handlers
    let add_owner_on_click = {
        let sel_ap = selected_apartment.clone();
        let owners_state = apartment_owners.clone();
        let message_state = message.clone();
        Callback::from(move |user_id: u64| {
            if let Some(aid) = *sel_ap {
                if owners_state.iter().any(|(id,_,_)| *id == user_id) { return; }
                let owners_state2 = owners_state.clone();
                let message2 = message_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let body = serde_json::json!({"user_id": user_id});
                    let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/apartments/{}/owners", aid))).header("Content-Type","application/json");
                    if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.body(body.to_string()).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(r2) = reqwasm::http::Request::get(&api_url(&format!("/api/v1/apartments/{}/owners", aid))).send().await { if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                                    let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("name")?.as_str()?.to_string(), v.get("email")?.as_str()?.to_string()))).collect();
                                    owners_state2.set(mapped);
                                }}
                            } else { message2.set(Some("Failed to add owner".into())); }
                        }
                        Err(_) => message2.set(Some("Network error".into())),
                    }
                });
            } else { message_state.set(Some("Select an apartment first".into())); }
        })
    };
    let remove_owner = {
        let sel_ap = selected_apartment.clone();
        let owners_state = apartment_owners.clone();
        Callback::from(move |owner_id: u64| {
            if let Some(aid) = *sel_ap {
                let owners_state2 = owners_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::delete(&api_url(&format!("/api/v1/apartments/{}/owners/{}", aid, owner_id)));
                    if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    if let Ok(resp) = req.send().await { if resp.ok() {
                        if let Ok(r2) = reqwasm::http::Request::get(&api_url(&format!("/api/v1/apartments/{}/owners", aid))).send().await { if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                            let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("name")?.as_str()?.to_string(), v.get("email")?.as_str()?.to_string()))).collect();
                            owners_state2.set(mapped);
                        }}
                    }}
                });
            }
        })
    };
    let filtered_users_vec: Vec<(u64,String,String)> = {
        let q = user_query.to_lowercase();
        (*all_users).iter().cloned().filter(|(_,n,e)| q.is_empty() || n.to_lowercase().contains(&q) || e.to_lowercase().contains(&q)).collect()
    };

    let modal_msg = if pending_delete_building.is_some() && pending_delete_apartment.is_some() { "Delete selected building and apartment?" }
        else if pending_delete_building.is_some() { "Delete selected building?" }
        else if pending_delete_apartment.is_some() { "Delete selected apartment?" } else { "" };
    let show_modal = pending_delete_building.is_some() || pending_delete_apartment.is_some();
    let modal = if show_modal { html!{
        <div class="modal fade show" style="display:block; background:rgba(0,0,0,.5);" role="dialog" aria-modal="true">
            <div class="modal-dialog modal-sm">
                <div class="modal-content">
                    <div class="modal-header">
                        <h6 class="modal-title">{"Confirm Deletion"}</h6>
                        <button type="button" class="btn-close" aria-label="Close" onclick={cancel_delete.clone()}></button>
                    </div>
                    <div class="modal-body"><p>{modal_msg}</p></div>
                    <div class="modal-footer">
                        <button class="btn btn-secondary btn-sm" onclick={cancel_delete.clone()}>{"Cancel"}</button>
                        { if pending_delete_building.is_some() { html!{<button class="btn btn-danger btn-sm" onclick={confirm_delete_building.clone()}>{"Delete Building"}</button>} } else { html!{} } }
                        { if pending_delete_apartment.is_some() { html!{<button class="btn btn-danger btn-sm" onclick={confirm_delete_apartment.clone()}>{"Delete Apartment"}</button>} } else { html!{} } }
                    </div>
                </div>
            </div>
        </div>
    }} else { html!{} };

    // effect: load deleted lists when toggled on
    {
        let show_deleted = show_deleted.clone();
        let del_b = deleted_buildings.clone();
        let del_a = deleted_apartments.clone();
        let loading_del = loading_deleted.clone();
        use_effect_with(show_deleted.clone(), move |_| {
            if *show_deleted {
                loading_del.set(true);
                let del_b2 = del_b.clone();
                let del_a2 = del_a.clone();
                let loading_del2 = loading_del.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    {
                        let mut req = reqwasm::http::Request::get(&api_url("/api/v1/buildings/deleted"));
                        if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                        if let Ok(resp) = req.send().await { if resp.ok() { if let Ok(list) = resp.json::<Vec<Building>>().await { del_b2.set(list); } } }
                    }
                    {
                        let mut req = reqwasm::http::Request::get(&api_url("/api/v1/apartments/deleted"));
                        if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                        if let Ok(resp) = req.send().await { if resp.ok() { if let Ok(list) = resp.json::<Vec<Apartment>>().await { del_a2.set(list); } } }
                    }
                    loading_del2.set(false);
                });
            } else { del_b.set(Vec::new()); del_a.set(Vec::new()); loading_del.set(false); }
            || ()
        });
    }

    let restore_building = {
        let buildings_state = buildings.clone();
        let deleted_buildings_state = deleted_buildings.clone();
        let show_deleted_state = show_deleted.clone();
        let message_state = message.clone();
        Callback::from(move |id: u64| {
            let buildings_state2 = buildings_state.clone();
            let deleted_buildings_state2 = deleted_buildings_state.clone();
            let show_deleted_state2 = show_deleted_state.clone();
            let message2 = message_state.clone();
            deleted_buildings_state2.set(deleted_buildings_state2.iter().cloned().filter(|b| b.id != id).collect());
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/buildings/{}/restore", id)));
                if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            let rb = reqwasm::http::Request::get(&api_url("/api/v1/buildings"));
                            if let Ok(r2) = rb.send().await { if r2.ok() { if let Ok(list) = r2.json::<Vec<Building>>().await { buildings_state2.set(list); } } }
                            if *show_deleted_state2 {
                                let mut rd = reqwasm::http::Request::get(&api_url("/api/v1/buildings/deleted"));
                                if let Some(tok) = crate::utils::auth::get_token() { rd = rd.header("Authorization", &format!("Bearer {}", tok)); }
                                if let Ok(r3) = rd.send().await { if r3.ok() { if let Ok(list) = r3.json::<Vec<Building>>().await { deleted_buildings_state2.set(list); } } }
                            }
                        } else { message2.set(Some("Restore failed".into())); }
                    }
                    Err(_) => message2.set(Some("Network error".into())),
                }
            });
        })
    };
    let restore_apartment = {
        let apartments_state = apartments.clone();
        let deleted_apartments_state = deleted_apartments.clone();
        let selected_building_state = selected_building.clone();
        let show_deleted_state = show_deleted.clone();
        let message_state = message.clone();
        Callback::from(move |id: u64| {
            let apartments_state2 = apartments_state.clone();
            let deleted_apartments_state2 = deleted_apartments_state.clone();
            let selected_building_state2 = selected_building_state.clone();
            let show_deleted_state2 = show_deleted_state.clone();
            let message2 = message_state.clone();
            deleted_apartments_state2.set(deleted_apartments_state2.iter().cloned().filter(|a| a.id != id).collect());
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/apartments/{}/restore", id)));
                if let Some(tok) = crate::utils::auth::get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Some(bid) = *selected_building_state2 { let url = format!("/api/v1/buildings/{}/apartments", bid); if let Ok(r2) = reqwasm::http::Request::get(&api_url(&url)).send().await { if r2.ok() { if let Ok(list) = r2.json::<Vec<Apartment>>().await { apartments_state2.set(list); } } } }
                            if *show_deleted_state2 { let mut rd = reqwasm::http::Request::get(&api_url("/api/v1/apartments/deleted")); if let Some(tok) = crate::utils::auth::get_token() { rd = rd.header("Authorization", &format!("Bearer {}", tok)); } if let Ok(r3) = rd.send().await { if r3.ok() { if let Ok(list) = r3.json::<Vec<Apartment>>().await { deleted_apartments_state2.set(list); } } } }
                        } else { message2.set(Some("Restore failed".into())); }
                    }
                    Err(_) => message2.set(Some("Network error".into())),
                }
            });
        })
    };

    let deleted_buildings_section: Html = {
        if *show_deleted {
            if *loading_deleted {
                html!{<div class="border rounded p-2 bg-light"><h6 class="mb-2 text-muted">{"Deleted Buildings"}</h6><Spinner center={true} /></div>}
            } else if deleted_buildings.is_empty() {
                html!{<div class="border rounded p-2 bg-light"><h6 class="mb-2 text-muted">{"Deleted Buildings"}</h6><div class="small text-muted">{"None"}</div></div>}
            } else {
                html!{<div class="border rounded p-2 bg-light"><h6 class="mb-2 text-muted">{"Deleted Buildings"}</h6><ul class="list-group list-group-sm">{ for deleted_buildings.iter().cloned().map(|b| { let rb = restore_building.clone(); html!{<li class="list-group-item py-1 d-flex justify-content-between align-items-center"><span>{b.address}</span><button class="btn btn-sm btn-outline-success" onclick={Callback::from(move |_| rb.emit(b.id))}>{"Restore"}</button></li>} }) }</ul></div>}
            }
        } else { html!{} }
    };
    let deleted_apartments_section: Html = {
        if *show_deleted {
            if *loading_deleted {
                html!{<div class="border rounded p-2 bg-light mt-3"><h6 class="mb-2 text-muted">{"Deleted Apartments"}</h6><Spinner center={true} /></div>}
            } else if deleted_apartments.is_empty() {
                html!{<div class="border rounded p-2 bg-light mt-3"><h6 class="mb-2 text-muted">{"Deleted Apartments"}</h6><div class="small text-muted">{"None"}</div></div>}
            } else {
                html!{<div class="border rounded p-2 bg-light mt-3"><h6 class="mb-2 text-muted">{"Deleted Apartments"}</h6><table class="table table-sm table-bordered mb-0"><thead><tr><th>{"ID"}</th><th>{"Number"}</th><th>{"Size"}</th><th></th></tr></thead><tbody>{ for deleted_apartments.iter().cloned().map(|a| { let ra = restore_apartment.clone(); html!{<tr><td>{a.id}</td><td>{a.number}</td><td>{a.size_sq_m.map(|s| format!("{:.1}", s)).unwrap_or("-".into())}</td><td><button class="btn btn-sm btn-outline-success" onclick={Callback::from(move |_| ra.emit(a.id))}>{"Restore"}</button></td></tr>} }) }</tbody></table></div>}
            }
        } else { html!{} }
    };
    // Simplified owners section to avoid deep nested braces
    let owners_section: Html = {
        match *selected_apartment {
            Some(aid) => {
                let sel_ap_handle = selected_apartment.clone();
                let owners_badges: Html = if *loading_owners {
                    html!{<Spinner center={true} />}
                } else {
                    html!{ for apartment_owners.iter().cloned().map(|(oid,name,_email)| { let rm=remove_owner.clone(); html!{<span class="badge bg-secondary me-1">{name} <button type="button" class="btn-close btn-close-white btn-sm" onclick={Callback::from(move |_| rm.emit(oid))}></button></span>} }) }
                };
                html!{
                    <div class="card mt-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{format!("Apartment {} Owners", aid)}</span>
                            <button class="btn btn-sm btn-outline-secondary" onclick={Callback::from(move |_| sel_ap_handle.set(None))}>{"Close"}</button>
                        </div>
                        <div class="card-body">
                            <div class="mb-2">{ owners_badges }</div>
                            <input class="form-control form-control-sm" placeholder="Search user" value={(*user_query).clone()} oninput={{ let q=user_query.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); q.set(i.value()); }) }} />
                            <div class="border rounded bg-light mt-2" style="max-height:160px; overflow:auto;">
                                { for filtered_users_vec.iter().cloned().map(|(id,name,email)| { let add_cb=add_owner_on_click.clone(); html!{<div class="px-2 py-1 small" style="cursor:pointer" onclick={Callback::from(move |_| add_cb.emit(id))}>{format!("{} – {}", name, email)}</div>} }) }
                            </div>
                            <p class="small text-muted mt-2">{"Click a user to assign as owner."}</p>
                        </div>
                    </div>
                }
            }
            None => html!{}
        }
    };

    html!{
        <div class="container mt-4">
            <h2>{"Manager"}</h2>
            // Announcements management section
            <div class="row mb-4">
                <div class="col-12">
                    <AnnouncementsManage />
                </div>
            </div>
            { if let Some(msg) = &*message { html!{<div class="alert alert-warning">{msg}</div>} } else { html!{} } }
            <div class="row">
                <div class="col-md-5">
                    <h5>{"Buildings"}</h5>
                    <div class="d-flex align-items-center mb-1">
                        <form class="row g-2 flex-grow-1" onsubmit={on_add_building.clone()}>
                            <div class="col-6"><input class="form-control form-control-sm" placeholder="Address" value={(*address).clone()} oninput={{ let a=address.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); a.set(i.value()); }) }} /></div>
                            <div class="col-3"><input class="form-control form-control-sm" placeholder="Year" value={(*year).clone()} oninput={{ let y=year.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); y.set(i.value()); }) }} /></div>
                            <div class="col-3"><button class="btn btn-sm btn-primary" type="submit">{"Add"}</button></div>
                        </form>
                    </div>
                    <ul class="list-group mb-2">{ if *loading_buildings && buildings.is_empty() { html!{<li class="list-group-item text-center"><Spinner center={true} /> </li>} } else { html!{ for buildings.iter().cloned().map(|b| {
                        let sel = selected_building.clone();
                        let pend = pending_delete_building.clone();
                        html!{<li class="list-group-item d-flex justify-content-between align-items-center" style="cursor:pointer" onclick={Callback::from(move |_| sel.set(Some(b.id)))}>
                            <span>{b.address.clone()}</span>
                            <button class="btn btn-outline-danger btn-sm" onclick={Callback::from(move |e: MouseEvent| { e.stop_propagation(); pend.set(Some(b.id)); })}>{"Delete"}</button>
                        </li>}
                    }) } } }</ul>
                    <div class="form-check form-switch mb-2">
                        <input class="form-check-input" type="checkbox" id="showDeletedSwitch" checked={*show_deleted} onchange={{ let sd=show_deleted.clone(); Callback::from(move |e: Event| { let cb: web_sys::HtmlInputElement = e.target_unchecked_into(); sd.set(cb.checked()); }) }} />
                        <label class="form-check-label" for="showDeletedSwitch">{"Show Deleted"}</label>
                    </div>
                    { deleted_buildings_section.clone() }
                </div>
                <div class="col-md-7">
                    <h5>{"Apartments"}</h5>
                    { if selected_building.is_some() { html!{
                        <form class="row g-2 mb-2" onsubmit={on_add_apartment.clone()}>
                            <div class="col-4"><input class="form-control form-control-sm" placeholder="Number" value={(*apt_number).clone()} oninput={{ let n=apt_number.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); n.set(i.value()); }) }} /></div>
                            <div class="col-4"><input class="form-control form-control-sm" placeholder="Size m2" value={(*apt_size).clone()} oninput={{ let s=apt_size.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.value()); }) }} /></div>
                            <div class="col-4"><button class="btn btn-sm btn-primary" type="submit">{"Add"}</button></div>
                        </form>
                    }} else { html!{<div class="alert alert-info py-1 px-2">{"Select a building to add apartments."}</div>} } }
                    <table class="table table-sm"><thead><tr><th>{"ID"}</th><th>{"Number"}</th><th>{"Size"}</th><th></th></tr></thead><tbody>{ if *loading_apartments && apartments.is_empty() { html!{<tr><td colspan="4" class="text-center"><Spinner center={true} /></td></tr>} } else { html!{ for apartments.iter().cloned().map(|a| { let ap_pending = pending_delete_apartment.clone(); let sel_ap_set = selected_apartment.clone(); html!{<tr onclick={Callback::from(move |_| sel_ap_set.set(Some(a.id)))} style="cursor:pointer"><td>{a.id}</td><td>{a.number}</td><td>{a.size_sq_m.map(|s| format!("{:.1}", s)).unwrap_or("-".into())}</td><td><button class="btn btn-outline-danger btn-sm" onclick={Callback::from(move |e: MouseEvent| { e.stop_propagation(); ap_pending.set(Some(a.id)); })}>{"Delete"}</button></td></tr>} }) } } }</tbody></table>
                    { owners_section.clone() }
                    { deleted_apartments_section.clone() }
                </div>
            </div>
            { modal.clone() }
        </div>
    }
}
