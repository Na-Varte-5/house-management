use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize, Clone, PartialEq)]
struct ProposalWithVotes {
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
    yes_count: i64,
    no_count: i64,
    abstain_count: i64,
    total_votes: i64,
    user_vote: Option<String>,
    user_eligible: bool,
    result: Option<ProposalResult>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct ProposalResult {
    id: u64,
    proposal_id: u64,
    passed: bool,
    yes_weight: String,
    no_weight: String,
    abstain_weight: String,
    total_weight: String,
    tallied_at: Option<String>,
    method_applied_version: String,
}

#[derive(Serialize)]
struct CastVotePayload {
    choice: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: u64,
}

#[function_component(VotingDetailPage)]
pub fn voting_detail_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let proposal = use_state(|| None::<ProposalWithVotes>);
    let loading = use_state(|| true);
    let voting = use_state(|| false);
    let tallying = use_state(|| false);

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let proposal_id = props.id;
    let token = auth.token().map(|t| t.to_string());

    // Load proposal
    {
        let proposal = proposal.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(proposal_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<ProposalWithVotes>(&format!("/proposals/{}", id)).await {
                    Ok(p) => {
                        proposal.set(Some(p));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load proposal: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_vote = {
        let proposal = proposal.clone();
        let voting = voting.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |choice: String| {
            let proposal = proposal.clone();
            let voting = voting.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            if let Some(p) = (*proposal).clone() {
                voting.set(true);
                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = CastVotePayload { choice: choice.clone() };

                    match client.post::<_, serde_json::Value>(&format!("/proposals/{}/vote", p.id), &payload).await {
                        Ok(_) => {
                            success.set(Some(format!("Vote cast: {}", choice)));
                            // Reload proposal to get updated counts
                            if let Ok(updated) = client.get::<ProposalWithVotes>(&format!("/proposals/{}", p.id)).await {
                                proposal.set(Some(updated));
                            }
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some("You are not eligible to vote on this proposal".to_string()));
                        }
                        Err(ApiError::BadRequest(msg)) => {
                            error.set(Some(msg));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to cast vote: {}", e)));
                        }
                    }
                    voting.set(false);
                });
            }
        })
    };

    let on_tally = {
        let proposal = proposal.clone();
        let tallying = tallying.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let proposal = proposal.clone();
            let tallying = tallying.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            if let Some(p) = (*proposal).clone() {
                tallying.set(true);
                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());

                    match client.post_empty::<serde_json::Value>(&format!("/proposals/{}/tally", p.id)).await {
                        Ok(_) => {
                            success.set(Some("Results tallied successfully".to_string()));
                            // Reload proposal to get results
                            if let Ok(updated) = client.get::<ProposalWithVotes>(&format!("/proposals/{}", p.id)).await {
                                proposal.set(Some(updated));
                            }
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some("You don't have permission to tally results".to_string()));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to tally results: {}", e)));
                        }
                    }
                    tallying.set(false);
                });
            }
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Voting);
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
            <div class="mb-3">
                <button class="btn btn-outline-secondary btn-sm" onclick={on_back}>
                    {"← Back to Proposals"}
                </button>
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
            } else if let Some(p) = (*proposal).clone() {
                <div class="row">
                    // Main proposal details
                    <div class="col-lg-8 mb-3">
                        <div class="card">
                            <div class="card-header">
                                <div class="d-flex justify-content-between align-items-start">
                                    <h4 class="mb-0">{&p.title}</h4>
                                    <span class={classes!("badge", status_class(&p.status))}>
                                        {&p.status}
                                    </span>
                                </div>
                            </div>
                            <div class="card-body">
                                <p>{&p.description}</p>

                                <hr />

                                <div class="row small">
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Voting Method:"}</strong>{" "}{&p.voting_method}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Eligible Voters:"}</strong>{" "}{&p.eligible_roles}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Start Time:"}</strong>{" "}{&p.start_time}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"End Time:"}</strong>{" "}{&p.end_time}
                                    </div>
                                </div>

                                <hr />

                                <h5>{"Vote Counts"}</h5>
                                <div class="row text-center mb-3">
                                    <div class="col-4">
                                        <div class="border rounded p-2 bg-success-subtle">
                                            <div class="fs-4 fw-bold text-success">{p.yes_count}</div>
                                            <div class="small">{"Yes"}</div>
                                        </div>
                                    </div>
                                    <div class="col-4">
                                        <div class="border rounded p-2 bg-danger-subtle">
                                            <div class="fs-4 fw-bold text-danger">{p.no_count}</div>
                                            <div class="small">{"No"}</div>
                                        </div>
                                    </div>
                                    <div class="col-4">
                                        <div class="border rounded p-2 bg-secondary-subtle">
                                            <div class="fs-4 fw-bold text-secondary">{p.abstain_count}</div>
                                            <div class="small">{"Abstain"}</div>
                                        </div>
                                    </div>
                                </div>
                                <div class="text-center text-muted small">
                                    {"Total votes: "}{p.total_votes}
                                </div>

                                {
                                    if let Some(user_vote) = &p.user_vote {
                                        html! {
                                            <div class="alert alert-info mt-3">
                                                {"Your current vote: "}<strong>{user_vote}</strong>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                {
                                    if let Some(result) = &p.result {
                                        html! {
                                            <>
                                                <hr />
                                                <h5>{"Final Results"}</h5>
                                                <div class={classes!("alert", if result.passed { "alert-success" } else { "alert-danger" })}>
                                                    <strong>
                                                        {if result.passed { "✓ Proposal PASSED" } else { "✗ Proposal FAILED" }}
                                                    </strong>
                                                </div>
                                                <div class="small">
                                                    <div>{"Yes Weight: "}{&result.yes_weight}</div>
                                                    <div>{"No Weight: "}{&result.no_weight}</div>
                                                    <div>{"Abstain Weight: "}{&result.abstain_weight}</div>
                                                    <div>{"Total Weight: "}{&result.total_weight}</div>
                                                    <div class="text-muted mt-2">
                                                        {"Tallied at: "}{result.tallied_at.as_ref().unwrap_or(&"(unknown)".to_string())}
                                                    </div>
                                                </div>
                                            </>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    </div>

                    // Voting panel
                    <div class="col-lg-4 mb-3">
                        {
                            if p.status == "Open" && p.user_eligible {
                                html! {
                                    <div class="card">
                                        <div class="card-header">
                                            <h5 class="mb-0">{"Cast Your Vote"}</h5>
                                        </div>
                                        <div class="card-body">
                                            <div class="d-grid gap-2">
                                                <button
                                                    class="btn btn-success"
                                                    disabled={*voting}
                                                    onclick={{
                                                        let on_vote = on_vote.clone();
                                                        Callback::from(move |_| on_vote.emit("Yes".to_string()))
                                                    }}
                                                >
                                                    if *voting {
                                                        <span class="spinner-border spinner-border-sm me-1"></span>
                                                    }
                                                    {"Vote Yes"}
                                                </button>
                                                <button
                                                    class="btn btn-danger"
                                                    disabled={*voting}
                                                    onclick={{
                                                        let on_vote = on_vote.clone();
                                                        Callback::from(move |_| on_vote.emit("No".to_string()))
                                                    }}
                                                >
                                                    if *voting {
                                                        <span class="spinner-border spinner-border-sm me-1"></span>
                                                    }
                                                    {"Vote No"}
                                                </button>
                                                <button
                                                    class="btn btn-secondary"
                                                    disabled={*voting}
                                                    onclick={{
                                                        let on_vote = on_vote.clone();
                                                        Callback::from(move |_| on_vote.emit("Abstain".to_string()))
                                                    }}
                                                >
                                                    if *voting {
                                                        <span class="spinner-border spinner-border-sm me-1"></span>
                                                    }
                                                    {"Abstain"}
                                                </button>
                                            </div>
                                            <div class="text-muted small mt-3">
                                                {"You can change your vote at any time before voting closes."}
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else if p.status == "Open" && !p.user_eligible {
                                html! {
                                    <div class="card">
                                        <div class="card-header">
                                            <h5 class="mb-0">{"Voting Information"}</h5>
                                        </div>
                                        <div class="card-body">
                                            <div class="alert alert-warning mb-0">
                                                <strong>{"Not Eligible"}</strong>
                                                <p class="mb-0 small">{"You are not eligible to vote on this proposal. Only members with the following roles can vote: "}{&p.eligible_roles}</p>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else if p.status == "Closed" && auth.is_admin_or_manager() && p.result.is_none() {
                                html! {
                                    <div class="card">
                                        <div class="card-header">
                                            <h5 class="mb-0">{"Management"}</h5>
                                        </div>
                                        <div class="card-body">
                                            <p class="small text-muted">{"Voting is closed. Tally the results to finalize the outcome."}</p>
                                            <button
                                                class="btn btn-primary w-100"
                                                disabled={*tallying}
                                                onclick={on_tally}
                                            >
                                                if *tallying {
                                                    <>
                                                        <span class="spinner-border spinner-border-sm me-1"></span>
                                                        {"Tallying..."}
                                                    </>
                                                } else {
                                                    {"Tally Results"}
                                                }
                                            </button>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="card">
                                        <div class="card-body">
                                            <p class="text-muted mb-0">
                                                {
                                                    match p.status.as_str() {
                                                        "Scheduled" => "Voting has not started yet.",
                                                        "Closed" => "Voting has closed.",
                                                        "Tallied" => "Results have been tallied.",
                                                        _ => "Voting is not available.",
                                                    }
                                                }
                                            </p>
                                        </div>
                                    </div>
                                }
                            }
                        }
                    </div>
                </div>
            } else {
                <div class="alert alert-warning">
                    {"Proposal not found"}
                </div>
            }
        </div>
    }
}
