use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

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
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    // Load proposals
    {
        let proposals = proposals.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Proposal>>("/proposals").await {
                    Ok(list) => {
                        proposals.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load proposals: {}", e)));
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

    // Status badge color
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
                <h2>{"Voting & Proposals"}</h2>
                { if auth.is_admin_or_manager() { html!{
                    <button class="btn btn-primary" onclick={on_new_proposal}>
                        {"+ New Proposal"}
                    </button>
                } } else { html!{} } }
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
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if proposals.is_empty() {
                <div class="alert alert-info">
                    {"No proposals found. Check back later for voting opportunities."}
                </div>
            } else {
                <div class="row">
                    {
                        for proposals.iter().map(|proposal| {
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
                                                                    <span class="badge bg-success" title="You can vote">
                                                                        {"✓ Can Vote"}
                                                                    </span>
                                                                }
                                                            } else {
                                                                html! {
                                                                    <span class="badge bg-secondary" title="View only - not eligible">
                                                                        {"View Only"}
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
                                                    {&proposal.voting_method}
                                                </span>
                                            </div>
                                            <div class="small text-muted">
                                                <div>{"Voting: "}{&proposal.start_time}{" - "}{&proposal.end_time}</div>
                                                <div>{"Eligible: "}{&proposal.eligible_roles}</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
            }
        </div>
    }
}
