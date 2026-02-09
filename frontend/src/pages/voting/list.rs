use crate::components::pagination::Pagination;
use crate::components::search_input::SearchInput;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::i18n::{t, t_with_args};
use crate::routes::Route;
use crate::services::api::{PaginatedResponse, PaginationMeta, api_client};
use crate::utils::datetime::format_dt_local;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

fn friendly_voting_method(method: &str) -> String {
    match method {
        "SimpleMajority" => t("voting-method-simple"),
        "WeightedArea" => t("voting-method-weighted"),
        "PerSeat" => t("voting-method-per-seat"),
        "Consensus" => t("voting-method-consensus"),
        other => other.to_string(),
    }
}

#[derive(Deserialize, Clone, PartialEq)]
struct Proposal {
    id: u64,
    title: String,
    description: String,
    created_by: u64,
    start_time: String,
    end_time: String,
    voting_method: String,
    eligible_roles: String,
    status: String,
    created_at: Option<String>,
}

impl Proposal {
    fn user_is_eligible(&self, auth: &AuthContext) -> bool {
        let eligible_roles: Vec<&str> = self.eligible_roles.split(',').collect();
        auth.has_any_role(&eligible_roles)
    }
}

#[function_component(VotingListPage)]
pub fn voting_list_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let proposals = use_state(|| Vec::<Proposal>::new());
    let pagination_meta = use_state(|| None::<PaginationMeta>);
    let current_page = use_state(|| 1i64);
    let search_query = use_state(String::default);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    {
        let proposals = proposals.clone();
        let pagination_meta = pagination_meta.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();
        let page = *current_page;

        use_effect_with(page, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let client = api_client(token.as_deref());
                let url = format!("/proposals?page={}&per_page=20", page);
                match client.get::<PaginatedResponse<Proposal>>(&url).await {
                    Ok(resp) => {
                        pagination_meta.set(Some(resp.pagination));
                        proposals.set(resp.data);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(t_with_args(
                            "voting-failed-load",
                            &[("error", &e.to_string())],
                        )));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_new_proposal = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::VotingNew);
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

    let on_page_change = {
        let current_page = current_page.clone();
        Callback::from(move |page: i64| {
            current_page.set(page);
        })
    };

    let on_search_change = {
        let search_query = search_query.clone();
        Callback::from(move |val: String| search_query.set(val))
    };

    let query_lower = search_query.to_lowercase();
    let filtered_proposals: Vec<&Proposal> = proposals
        .iter()
        .filter(|p| {
            query_lower.is_empty()
                || p.title.to_lowercase().contains(&query_lower)
                || p.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    let status_class = |status: &str| match status {
        "Scheduled" => "bg-secondary",
        "Open" => "bg-success",
        "Closed" => "bg-warning text-dark",
        "Tallied" => "bg-primary",
        _ => "bg-light text-dark",
    };

    html! {
        <div class="container mt-4">
            <div class="d-flex justify-content-between align-items-center mb-3">
                <h2>{t("voting-title")}</h2>
                { if auth.is_admin_or_manager() { html!{
                    <button class="btn btn-primary" onclick={on_new_proposal}>
                        {t("voting-new-proposal")}
                    </button>
                } } else { html!{} } }
            </div>

            <div class="mb-3">
                <SearchInput
                    value={(*search_query).clone()}
                    on_change={on_search_change}
                    placeholder={t("voting-search-placeholder")}
                />
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{t("voting-loading")}</span>
                    </div>
                </div>
            } else if filtered_proposals.is_empty() {
                <div class="alert alert-info">
                    {
                        if search_query.is_empty() {
                            t("voting-no-proposals-check-later")
                        } else {
                            t("voting-no-proposals-match")
                        }
                    }
                </div>
            } else {
                <div class="row">
                    {
                        for filtered_proposals.iter().map(|proposal| {
                            let id = proposal.id;
                            let navigator = navigator.clone();
                            let is_eligible = proposal.user_is_eligible(&auth);
                            html! {
                                <div class="col-md-6 col-lg-4 mb-3">
                                    <div
                                        class="card h-100"
                                        style="cursor: pointer;"
                                        onclick={{
                                            Callback::from(move |_| {
                                                navigator.push(&Route::VotingDetail { id });
                                            })
                                        }}
                                    >
                                        <div class="card-body">
                                            <div class="d-flex justify-content-between align-items-start mb-2">
                                                <h5 class="card-title mb-0">{&proposal.title}</h5>
                                                <div class="d-flex gap-1">
                                                    {
                                                        if proposal.status == "Open" {
                                                            if is_eligible {
                                                                html! {
                                                                    <span class="badge bg-success" title={t("voting-can-vote")}>
                                                                        {t("voting-can-vote")}
                                                                    </span>
                                                                }
                                                            } else {
                                                                html! {
                                                                    <span class="badge bg-secondary" title={t("voting-view-only")}>
                                                                        {t("voting-view-only")}
                                                                    </span>
                                                                }
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <span class={classes!("badge", status_class(&proposal.status))}>
                                                        {&proposal.status}
                                                    </span>
                                                </div>
                                            </div>
                                            <p class="card-text text-muted small mb-2">
                                                {&proposal.description}
                                            </p>
                                            <div class="mb-2">
                                                <span class="badge bg-light text-dark">
                                                    {friendly_voting_method(&proposal.voting_method)}
                                                </span>
                                            </div>
                                            <div class="small text-muted">
                                                <div>{t("voting-voting-label")}{" "}{format_dt_local(&proposal.start_time)}{" — "}{format_dt_local(&proposal.end_time)}</div>
                                                <div>{t("voting-eligible-label")}{" "}{&proposal.eligible_roles}</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>

                if let Some(ref m) = *pagination_meta {
                    <Pagination
                        current_page={m.page}
                        total_pages={m.total_pages}
                        total_items={m.total}
                        on_page_change={on_page_change.clone()}
                    />
                }
            }
        </div>
    }
}
