use crate::models::{Proposal, ProposalResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Get a single proposal with vote counts
#[derive(Serialize, ToSchema)]
pub struct ProposalWithVotes {
    #[serde(flatten)]
    pub proposal: Proposal,
    pub yes_count: i64,
    pub no_count: i64,
    pub abstain_count: i64,
    pub total_votes: i64,
    pub user_vote: Option<String>,
    pub user_eligible: bool,
    pub result: Option<ProposalResult>,
}

/// Create a new proposal (Admin/Manager only)
#[derive(Deserialize, ToSchema)]
pub struct CreateProposalPayload {
    pub title: String,
    pub description: String,
    pub building_id: Option<u64>, // If None, proposal is global (visible to all)
    #[schema(example = "2026-01-20T10:00")]
    pub start_time: String, // ISO datetime string (YYYY-MM-DDTHH:MM)
    #[schema(example = "2026-01-27T18:00")]
    pub end_time: String, // ISO datetime string (YYYY-MM-DDTHH:MM)
    #[schema(example = "SimpleMajority")]
    pub voting_method: String,
    pub eligible_roles: Vec<String>,
}

/// Cast a vote on a proposal
#[derive(Deserialize, ToSchema)]
pub struct CastVotePayload {
    #[schema(example = "Yes")]
    pub choice: String, // "Yes", "No", "Abstain"
}
