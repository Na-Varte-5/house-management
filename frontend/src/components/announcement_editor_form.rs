use crate::components::spinner::Spinner;
use crate::i18n::t;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AnnouncementEditorFormProps {
    // Form data
    pub title: String,
    pub body_md: String,
    pub public_flag: bool,
    pub pinned_flag: bool,
    pub comments_enabled: bool,
    pub publish_at: String,
    pub expire_at: String,
    pub selected_roles: Vec<String>,
    pub selected_building: Option<u64>,
    pub selected_apartment: Option<u64>,
    pub buildings: Vec<(u64, String)>,
    pub apartments: Vec<(u64, u64, String)>,
    pub preview_html: String,

    // UI state
    pub saving: bool,
    pub error: Option<String>,
    pub is_editing: bool,
    pub publish_now_id: Option<u64>,

    // Callbacks
    pub on_title_change: Callback<String>,
    pub on_body_md_change: Callback<String>,
    pub on_public_change: Callback<bool>,
    pub on_pinned_change: Callback<bool>,
    pub on_comments_change: Callback<bool>,
    pub on_publish_at_change: Callback<String>,
    pub on_expire_at_change: Callback<String>,
    pub on_roles_change: Callback<Vec<String>>,
    pub on_building_change: Callback<Option<u64>>,
    pub on_apartment_change: Callback<Option<u64>>,
    pub on_submit: Callback<SubmitEvent>,
    pub on_publish_now: Callback<u64>,
    pub on_cancel: Callback<()>,
}

#[function_component(AnnouncementEditorForm)]
pub fn announcement_editor_form(props: &AnnouncementEditorFormProps) -> Html {
    let active_tab = use_state(|| "edit".to_string());

    html! {
        <div class="card mb-4">
            <div class="card-header">
                <strong>
                    { if props.is_editing {
                        t("button-update")
                    } else {
                        t("button-publish")
                    } }
                </strong>
            </div>
            <div class="card-body">
                { if let Some(err) = &props.error {
                    html!{<div class="alert alert-danger py-1">{err}</div>}
                } else {
                    html!{<></>}
                } }
                <form onsubmit={props.on_submit.clone()}>
                    <div class="mb-2">
                        <label class="form-label small">{ t("announcement-title-label") }</label>
                        <input
                            class="form-control form-control-sm"
                            value={props.title.clone()}
                            oninput={{
                                let cb = props.on_title_change.clone();
                                Callback::from(move |e: InputEvent| {
                                    let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    cb.emit(i.value());
                                })
                            }}
                            placeholder={t("announcement-title-label")}
                        />
                    </div>

                    // Edit/Preview tabs
                    <div class="mb-3">
                        <ul class="nav nav-tabs nav-sm">
                            <li class="nav-item">
                                <button
                                    type="button"
                                    class={classes!("nav-link", if *active_tab=="edit" {"active"} else {""})}
                                    onclick={{ let tab=active_tab.clone(); Callback::from(move |_| tab.set("edit".into())) }}
                                >
                                    { t("announcement-edit-tab") }
                                </button>
                            </li>
                            <li class="nav-item">
                                <button
                                    type="button"
                                    class={classes!("nav-link", if *active_tab=="preview" {"active"} else {""})}
                                    onclick={{ let tab=active_tab.clone(); Callback::from(move |_| tab.set("preview".into())) }}
                                >
                                    { t("announcement-preview-tab") }
                                </button>
                            </li>
                        </ul>
                    </div>

                    // Body editor/preview
                    <div class="row">
                        { if *active_tab == "edit" {
                            html!{
                                <div class="col-12">
                                    <label class="form-label small">{ t("announcement-body-label") }</label>
                                    <textarea
                                        class="form-control"
                                        rows=12
                                        value={props.body_md.clone()}
                                        oninput={{
                                            let cb = props.on_body_md_change.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let i: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                                cb.emit(i.value());
                                            })
                                        }}
                                    />
                                    <div class="form-text small text-muted">
                                        { t("announcement-edit-tab") }{" -> "}{ t("announcement-preview-tab") }
                                    </div>
                                </div>
                            }
                        } else {
                            html!{
                                <div class="col-12">
                                    <label class="form-label small d-flex justify-content-between align-items-center">
                                        { t("announcement-preview-tab") }
                                        <span class="badge bg-secondary">{ t("preview-rendered") }</span>
                                    </label>
                                    <div class="alert alert-secondary mb-0 p-0 border">
                                        <div class="preview-body p-3" style="min-height:240px">
                                            { Html::from_html_unchecked(props.preview_html.clone().into()) }
                                        </div>
                                    </div>
                                </div>
                            }
                        } }
                    </div>

                    // Additional fields: roles, dates, building/apartment, options
                    <div class="row mt-2 g-2">
                        // Roles column
                        <div class="col-md-4">
                            <label class="form-label small">{ t("announcement-roles-label") }{" (visibility)"}</label>
                            { for ["Admin", "Manager", "Homeowner", "Renter", "HOA Member"].iter().map(|role| {
                                let role_str = role.to_string();
                                let checked = props.selected_roles.iter().any(|r| r == role);
                                let cb = props.on_roles_change.clone();
                                let current_roles = props.selected_roles.clone();
                                let role_for_change = role_str.clone();

                                let label_key = match *role {
                                    "Admin" => "role-admin",
                                    "Manager" => "role-manager",
                                    "Homeowner" => "role-homeowner",
                                    "Renter" => "role-renter",
                                    "HOA Member" => "role-hoa-member",
                                    _ => "role-unknown",
                                };

                                html!{
                                    <div class="form-check form-check-sm">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id={format!("role_{}", role.replace(' ', "_"))}
                                            checked={checked}
                                            onchange={Callback::from(move |e: Event| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                let mut list = current_roles.clone();
                                                if input.checked() {
                                                    if !list.contains(&role_for_change) {
                                                        list.push(role_for_change.clone());
                                                    }
                                                } else {
                                                    list.retain(|r| r != &role_for_change);
                                                }
                                                cb.emit(list);
                                            })}
                                        />
                                        <label class="form-check-label small" for={format!("role_{}", role.replace(' ', "_"))}>
                                            { t(label_key) }
                                        </label>
                                    </div>
                                }
                            }) }
                            <div class="form-text small text-muted">{ t("announcement-roles-help") }</div>
                        </div>

                        // Publish/Expire dates
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-publish-at-label") }</label>
                            <input
                                type="datetime-local"
                                class="form-control form-control-sm"
                                value={props.publish_at.clone()}
                                oninput={{
                                    let cb = props.on_publish_at_change.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        cb.emit(i.value());
                                    })
                                }}
                            />
                        </div>
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-expire-at-label") }</label>
                            <input
                                type="datetime-local"
                                class="form-control form-control-sm"
                                value={props.expire_at.clone()}
                                oninput={{
                                    let cb = props.on_expire_at_change.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        cb.emit(i.value());
                                    })
                                }}
                            />
                        </div>

                        // Building/Apartment selectors
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-building-label") }{" (optional)"}</label>
                            <select
                                class="form-select form-select-sm"
                                onchange={{
                                    let cb_building = props.on_building_change.clone();
                                    let cb_apartment = props.on_apartment_change.clone();
                                    Callback::from(move |e: Event| {
                                        let sel: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        let val = sel.value();
                                        if val.is_empty() {
                                            cb_building.emit(None);
                                            cb_apartment.emit(None);
                                        } else if let Ok(parsed) = val.parse::<u64>() {
                                            cb_building.emit(Some(parsed));
                                            cb_apartment.emit(None);
                                        }
                                    })
                                }}
                            >
                                <option value="" selected={props.selected_building.is_none()}>
                                    { t("none-option") }
                                </option>
                                { for props.buildings.iter().map(|(id, addr)| {
                                    let selected = props.selected_building == Some(*id);
                                    html!{
                                        <option value={id.to_string()} selected={selected}>
                                            {addr}
                                        </option>
                                    }
                                }) }
                            </select>

                            <label class="form-label small mt-2">{ t("announcement-apartment-label") }</label>
                            <select
                                class="form-select form-select-sm"
                                disabled={props.selected_building.is_none() || props.apartments.is_empty()}
                                onchange={{
                                    let cb = props.on_apartment_change.clone();
                                    Callback::from(move |e: Event| {
                                        let sel: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        let val = sel.value();
                                        if val.is_empty() {
                                            cb.emit(None);
                                        } else if let Ok(parsed) = val.parse::<u64>() {
                                            cb.emit(Some(parsed));
                                        }
                                    })
                                }}
                            >
                                <option value="" selected={props.selected_apartment.is_none()}>
                                    { t("none-option") }
                                </option>
                                { for props.apartments.iter()
                                    .filter(|(_, bid, _)| Some(*bid) == props.selected_building)
                                    .map(|(id, _, num)| {
                                        let selected = props.selected_apartment == Some(*id);
                                        html!{
                                            <option value={id.to_string()} selected={selected}>
                                                {num}
                                            </option>
                                        }
                                    })
                                }
                            </select>
                        </div>

                        // Options: public, pinned, comments
                        <div class="col-md-2">
                            <label class="form-label small">{ t("announcement-options-heading") }</label>
                            <div class="form-check">
                                <input
                                    class="form-check-input"
                                    type="checkbox"
                                    checked={props.public_flag}
                                    onchange={{
                                        let cb = props.on_public_change.clone();
                                        Callback::from(move |e: Event| {
                                            let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            cb.emit(i.checked());
                                        })
                                    }}
                                    id="annPublic"
                                />
                                <label class="form-check-label small" for="annPublic">
                                    { t("announcement-public-label") }
                                </label>
                            </div>
                            <div class="form-check">
                                <input
                                    class="form-check-input"
                                    type="checkbox"
                                    checked={props.pinned_flag}
                                    onchange={{
                                        let cb = props.on_pinned_change.clone();
                                        Callback::from(move |e: Event| {
                                            let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            cb.emit(i.checked());
                                        })
                                    }}
                                    id="annPinned"
                                />
                                <label class="form-check-label small" for="annPinned">
                                    { t("announcement-pin-label") }
                                </label>
                            </div>
                            <div class="form-check">
                                <input
                                    class="form-check-input"
                                    type="checkbox"
                                    checked={props.comments_enabled}
                                    onchange={{
                                        let cb = props.on_comments_change.clone();
                                        Callback::from(move |e: Event| {
                                            let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            cb.emit(i.checked());
                                        })
                                    }}
                                    id="annComments"
                                />
                                <label class="form-check-label small" for="annComments">
                                    { t("announcement-comments-enabled-label") }
                                </label>
                            </div>
                        </div>
                    </div>

                    // Action buttons
                    <div class="mt-3 d-flex gap-2">
                        <button class="btn btn-sm btn-primary" type="submit" disabled={props.saving}>
                            { if props.saving {
                                html!{<Spinner small={true} color={"light"} />}
                            } else if props.is_editing {
                                html!{ t("button-save") }
                            } else {
                                html!{ t("button-publish") }
                            } }
                        </button>
                        { if let Some(id) = props.publish_now_id {
                            let cb = props.on_publish_now.clone();
                            html!{
                                <button
                                    type="button"
                                    class="btn btn-sm btn-success"
                                    disabled={props.saving}
                                    onclick={Callback::from(move |_| cb.emit(id))}
                                >
                                    { t("announcement-publish-now") }
                                </button>
                            }
                        } else {
                            html!{}
                        } }
                        { if props.is_editing {
                            let cb = props.on_cancel.clone();
                            html!{
                                <button
                                    type="button"
                                    class="btn btn-sm btn-outline-secondary"
                                    onclick={Callback::from(move |_| cb.emit(()))}
                                >
                                    { t("button-cancel") }
                                </button>
                            }
                        } else {
                            html!{}
                        } }
                    </div>
                </form>
            </div>
        </div>
    }
}
