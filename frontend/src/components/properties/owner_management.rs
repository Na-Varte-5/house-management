use super::types::UserInfo;
use crate::i18n::t;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct OwnerManagementProps {
    pub owners: Vec<UserInfo>,
    pub all_users: Vec<UserInfo>,
    pub user_query: String,
    pub on_query_change: Callback<String>,
    pub on_assign: Callback<u64>,
    pub on_remove: Callback<u64>,
    pub loading: bool,
    pub show: bool,
}

#[function_component(OwnerManagement)]
pub fn owner_management(props: &OwnerManagementProps) -> Html {
    if !props.show {
        return html! {
            <div class="alert alert-info small mb-0">
                <i class="bi bi-info-circle me-2"></i>
                {t("properties-select-apartment")}
            </div>
        };
    }

    // Filter users based on query
    let filtered_users: Vec<&UserInfo> = if props.user_query.is_empty() {
        props.all_users.iter().collect()
    } else {
        let query_lower = props.user_query.to_lowercase();
        props
            .all_users
            .iter()
            .filter(|u| {
                u.name.to_lowercase().contains(&query_lower)
                    || u.email.to_lowercase().contains(&query_lower)
            })
            .collect()
    };

    // Exclude already assigned owners
    let owner_ids: Vec<u64> = props.owners.iter().map(|o| o.id).collect();
    let available_users: Vec<&UserInfo> = filtered_users
        .into_iter()
        .filter(|u| !owner_ids.contains(&u.id))
        .take(5)
        .collect();

    html! {
        <div>
            <h6 class="small fw-semibold mb-2">{t("properties-current-owners")}</h6>
            if props.loading {
                <div class="text-center py-2">
                    <div class="spinner-border spinner-border-sm" role="status">
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            } else if props.owners.is_empty() {
                <div class="alert alert-warning small mb-3">
                    {t("properties-no-owners")}
                </div>
            } else {
                <ul class="list-group list-group-flush mb-3">
                    { for props.owners.iter().map(|owner| {
                        let owner_id = owner.id;
                        let on_remove = {
                            let on_remove_cb = props.on_remove.clone();
                            Callback::from(move |_| on_remove_cb.emit(owner_id))
                        };

                        html! {
                            <li class="list-group-item d-flex justify-content-between align-items-center px-0">
                                <div>
                                    <strong class="small">{&owner.name}</strong>
                                    <br/>
                                    <span class="text-muted" style="font-size: 0.75rem;">{&owner.email}</span>
                                </div>
                                <button
                                    class="btn btn-sm btn-outline-danger"
                                    onclick={on_remove}
                                    title="Remove owner"
                                >
                                    <i class="bi bi-x-circle"></i>
                                </button>
                            </li>
                        }
                    }) }
                </ul>
            }

            <h6 class="small fw-semibold mb-2">{t("properties-assign-owner")}</h6>
            <input
                type="text"
                class="form-control form-control-sm mb-2"
                placeholder={t("properties-search-users")}
                value={props.user_query.clone()}
                oninput={{
                    let on_query_change = props.on_query_change.clone();
                    Callback::from(move |e: InputEvent| {
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        on_query_change.emit(input.value());
                    })
                }}
            />

            if !props.user_query.is_empty() && !available_users.is_empty() {
                <ul class="list-group list-group-flush">
                    { for available_users.iter().map(|user| {
                        let user_id = user.id;
                        let on_assign = {
                            let on_assign_cb = props.on_assign.clone();
                            Callback::from(move |_| on_assign_cb.emit(user_id))
                        };

                        html! {
                            <li
                                class="list-group-item list-group-item-action px-2"
                                onclick={on_assign}
                                style="cursor: pointer;"
                            >
                                <div class="small">
                                    <strong>{&user.name}</strong>
                                    <br/>
                                    <span class="text-muted" style="font-size: 0.75rem;">{&user.email}</span>
                                </div>
                            </li>
                        }
                    }) }
                </ul>
            } else if !props.user_query.is_empty() && available_users.is_empty() {
                <div class="alert alert-info small mb-0">
                    {t("properties-no-matching-users")}
                </div>
            }
        </div>
    }
}
