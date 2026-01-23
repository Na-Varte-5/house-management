use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{NewProposal, Proposal, ProposalResult, Vote, VoteChoice, VotingMethod};
use actix_web::{HttpResponse, Responder, web};
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::prelude::*;
use std::str::FromStr;
use utoipa;

/// List all proposals
///
/// Returns all proposals ordered by creation date (most recent first).
/// Accessible to all authenticated users.
#[utoipa::path(
    get,
    path = "/api/v1/proposals",
    responses(
        (status = 200, description = "List of proposals", body = Vec<Proposal>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Voting",
    security(("bearer_auth" = []))
)]
pub async fn list_proposals(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::auth::get_user_building_ids;
    use crate::schema::proposals::dsl as p;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get user's accessible buildings
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin = auth.has_any_role(&["Admin"]);
    let building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    // Build query
    let mut query = p::proposals.into_boxed();

    // If user has restricted access, filter by accessible buildings OR global proposals
    if let Some(ref ids) = building_ids {
        query = query.filter(p::building_id.eq_any(ids).or(p::building_id.is_null()));
    }

    let proposals = query
        .select(Proposal::as_select())
        .order(p::created_at.desc())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(proposals))
}

/// Get a single proposal with vote counts
#[derive(serde::Serialize, utoipa::ToSchema)]
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

/// Get proposal details with vote statistics
///
/// Returns detailed information about a proposal including vote counts,
/// whether the current user has voted, and if they are eligible to vote.
#[utoipa::path(
    get,
    path = "/api/v1/proposals/{id}",
    params(
        ("id" = u64, Path, description = "Proposal ID")
    ),
    responses(
        (status = 200, description = "Proposal details with vote statistics", body = ProposalWithVotes),
        (status = 404, description = "Proposal not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Voting",
    security(("bearer_auth" = []))
)]
pub async fn get_proposal(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::proposal_results::dsl as pr;
    use crate::schema::proposals::dsl as p;
    use crate::schema::votes::dsl as v;

    let id = path.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let proposal: Proposal = p::proposals
        .filter(p::id.eq(id))
        .select(Proposal::as_select())
        .first(&mut conn)?;

    // Count votes by choice
    let votes: Vec<Vote> = v::votes
        .filter(v::proposal_id.eq(id))
        .select(Vote::as_select())
        .load(&mut conn)?;

    let yes_count = votes.iter().filter(|v| v.choice == "Yes").count() as i64;
    let no_count = votes.iter().filter(|v| v.choice == "No").count() as i64;
    let abstain_count = votes.iter().filter(|v| v.choice == "Abstain").count() as i64;
    let total_votes = votes.len() as i64;

    // Check if user has voted
    let user_vote = votes
        .iter()
        .find(|v| v.user_id == user_id)
        .map(|v| v.choice.clone());

    // Check if user is eligible to vote
    let eligible_roles: Vec<&str> = proposal.eligible_roles.split(',').collect();
    let user_eligible = auth.has_any_role(&eligible_roles);

    // Get result if tallied
    let result: Option<ProposalResult> = pr::proposal_results
        .filter(pr::proposal_id.eq(id))
        .select(ProposalResult::as_select())
        .first(&mut conn)
        .ok();

    Ok(HttpResponse::Ok().json(ProposalWithVotes {
        proposal,
        yes_count,
        no_count,
        abstain_count,
        total_votes,
        user_vote,
        user_eligible,
        result,
    }))
}

/// Create a new proposal (Admin/Manager only)
#[derive(serde::Deserialize, utoipa::ToSchema)]
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

/// Create a new proposal
///
/// Creates a new voting proposal. Only Admin or Manager roles can create proposals.
/// The proposal status is automatically determined based on start/end times.
#[utoipa::path(
    post,
    path = "/api/v1/proposals",
    request_body = CreateProposalPayload,
    responses(
        (status = 201, description = "Proposal created successfully", body = Proposal),
        (status = 400, description = "Invalid input (e.g., invalid datetime format or voting method)"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Voting",
    security(("bearer_auth" = []))
)]
pub async fn create_proposal(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    payload: web::Json<CreateProposalPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::proposals::dsl as p;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let created_by = auth.claims.sub.parse::<u64>().unwrap_or(0);

    // Parse datetimes
    let start_time = chrono::NaiveDateTime::parse_from_str(&payload.start_time, "%Y-%m-%dT%H:%M")
        .map_err(|_| AppError::BadRequest("Invalid start_time format".into()))?;
    let end_time = chrono::NaiveDateTime::parse_from_str(&payload.end_time, "%Y-%m-%dT%H:%M")
        .map_err(|_| AppError::BadRequest("Invalid end_time format".into()))?;

    // Validate voting method
    VotingMethod::from_str(&payload.voting_method)
        .map_err(|_| AppError::BadRequest("Invalid voting_method".into()))?;

    // Validate building access (if building_id is specified)
    if let Some(building_id) = payload.building_id {
        use crate::auth::get_user_building_ids;
        let is_admin = auth.has_any_role(&["Admin"]);
        let accessible_buildings = get_user_building_ids(created_by, is_admin, &mut conn)?;

        // If user has restricted access, verify they can access this building
        if let Some(buildings) = accessible_buildings
            && !buildings.contains(&building_id)
        {
            return Err(AppError::Forbidden);
        }
    }

    let eligible_roles = payload.eligible_roles.join(",");

    // Determine status based on start time
    let now = chrono::Local::now().naive_local();
    let status = if start_time > now {
        "Scheduled"
    } else if end_time < now {
        "Closed"
    } else {
        "Open"
    };

    let new_proposal = NewProposal {
        title: payload.title.clone(),
        description: payload.description.clone(),
        building_id: payload.building_id,
        start_time,
        end_time,
        voting_method: payload.voting_method.clone(),
        eligible_roles,
        status: status.to_string(),
    };

    diesel::insert_into(p::proposals)
        .values((
            p::title.eq(new_proposal.title),
            p::description.eq(new_proposal.description),
            p::created_by.eq(created_by),
            p::start_time.eq(new_proposal.start_time),
            p::end_time.eq(new_proposal.end_time),
            p::voting_method.eq(new_proposal.voting_method),
            p::eligible_roles.eq(new_proposal.eligible_roles),
            p::status.eq(new_proposal.status),
        ))
        .execute(&mut conn)?;

    // Get the last inserted ID
    let inserted_id: u64 = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
    >("LAST_INSERT_ID()"))
    .first(&mut conn)?;

    // Fetch and return the created proposal
    let created_proposal: Proposal = p::proposals
        .filter(p::id.eq(inserted_id))
        .select(Proposal::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Created().json(created_proposal))
}

/// Cast a vote on a proposal
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CastVotePayload {
    #[schema(example = "Yes")]
    pub choice: String, // "Yes", "No", "Abstain"
}

/// Cast or update a vote on a proposal
///
/// Allows eligible users to vote on an open proposal. If the user has already voted,
/// this endpoint updates their existing vote. Vote weight is calculated based on the
/// proposal's voting method (SimpleMajority, WeightedArea, PerSeat, or Consensus).
#[utoipa::path(
    post,
    path = "/api/v1/proposals/{id}/vote",
    params(
        ("id" = u64, Path, description = "Proposal ID")
    ),
    request_body = CastVotePayload,
    responses(
        (status = 200, description = "Vote cast successfully"),
        (status = 400, description = "Invalid choice or proposal not open for voting"),
        (status = 403, description = "Forbidden - user not eligible to vote on this proposal"),
        (status = 404, description = "Proposal not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Voting",
    security(("bearer_auth" = []))
)]
pub async fn cast_vote(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<CastVotePayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::proposals::dsl as p;
    use crate::schema::votes::dsl as v;

    let proposal_id = path.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Validate vote choice
    VoteChoice::from_str(&payload.choice)
        .map_err(|_| AppError::BadRequest("Invalid choice".into()))?;

    // Get proposal
    let proposal: Proposal = p::proposals
        .filter(p::id.eq(proposal_id))
        .select(Proposal::as_select())
        .first(&mut conn)?;

    // Check if proposal is open
    if proposal.status != "Open" {
        return Err(AppError::BadRequest(
            "Proposal is not open for voting".into(),
        ));
    }

    // Check if user is eligible
    let eligible_roles: Vec<&str> = proposal.eligible_roles.split(',').collect();
    if !auth.has_any_role(&eligible_roles) {
        return Err(AppError::Forbidden);
    }

    // Calculate vote weight based on voting method
    let weight = match VotingMethod::from_str(&proposal.voting_method).unwrap() {
        VotingMethod::SimpleMajority | VotingMethod::PerSeat => BigDecimal::from_f64(1.0).unwrap(),
        VotingMethod::WeightedArea => {
            // Weight by total apartment area owned by user
            let areas: Vec<Option<f64>> = ao::apartment_owners
                .filter(ao::user_id.eq(user_id))
                .inner_join(apt::apartments.on(apt::id.eq(ao::apartment_id)))
                .select(apt::size_sq_m)
                .load(&mut conn)?;

            let total: f64 = areas.into_iter().flatten().sum();
            BigDecimal::from_f64(total).unwrap_or_else(|| BigDecimal::from_f64(0.0).unwrap())
        }
        VotingMethod::Consensus => BigDecimal::from_f64(1.0).unwrap(),
    };

    // Check if vote already exists
    let existing_vote: Option<Vote> = v::votes
        .filter(v::proposal_id.eq(proposal_id))
        .filter(v::user_id.eq(user_id))
        .select(Vote::as_select())
        .first(&mut conn)
        .ok();

    if let Some(existing) = existing_vote {
        // Update existing vote
        diesel::update(v::votes.filter(v::id.eq(existing.id)))
            .set((v::weight_decimal.eq(&weight), v::choice.eq(&payload.choice)))
            .execute(&mut conn)?;
    } else {
        // Insert new vote
        diesel::insert_into(v::votes)
            .values((
                v::proposal_id.eq(proposal_id),
                v::user_id.eq(user_id),
                v::weight_decimal.eq(&weight),
                v::choice.eq(&payload.choice),
            ))
            .execute(&mut conn)?;
    }

    #[derive(serde::Serialize)]
    struct VoteResponse {
        success: bool,
        choice: String,
    }

    Ok(HttpResponse::Ok().json(VoteResponse {
        success: true,
        choice: payload.choice.clone(),
    }))
}

/// Tally results for a proposal
///
/// Calculates and stores the final results for a proposal. Only Admin or Manager roles
/// can tally results. This determines whether the proposal passed based on the voting
/// method and updates the proposal status to "Tallied".
#[utoipa::path(
    post,
    path = "/api/v1/proposals/{id}/tally",
    params(
        ("id" = u64, Path, description = "Proposal ID")
    ),
    responses(
        (status = 200, description = "Results tallied successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Proposal not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Voting",
    security(("bearer_auth" = []))
)]
pub async fn tally_results(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::proposal_results::dsl as pr;
    use crate::schema::proposals::dsl as p;
    use crate::schema::votes::dsl as v;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let proposal_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get proposal
    let proposal: Proposal = p::proposals
        .filter(p::id.eq(proposal_id))
        .select(Proposal::as_select())
        .first(&mut conn)?;

    // Get all votes
    let votes: Vec<Vote> = v::votes
        .filter(v::proposal_id.eq(proposal_id))
        .select(Vote::as_select())
        .load(&mut conn)?;

    // Calculate totals
    let yes_weight: BigDecimal = votes
        .iter()
        .filter(|v| v.choice == "Yes")
        .map(|v| v.weight_decimal.clone())
        .sum();

    let no_weight: BigDecimal = votes
        .iter()
        .filter(|v| v.choice == "No")
        .map(|v| v.weight_decimal.clone())
        .sum();

    let abstain_weight: BigDecimal = votes
        .iter()
        .filter(|v| v.choice == "Abstain")
        .map(|v| v.weight_decimal.clone())
        .sum();

    let total_weight = yes_weight.clone() + no_weight.clone() + abstain_weight.clone();

    // Determine if passed based on voting method
    let passed = match VotingMethod::from_str(&proposal.voting_method).unwrap() {
        VotingMethod::SimpleMajority | VotingMethod::WeightedArea | VotingMethod::PerSeat => {
            yes_weight > no_weight
        }
        VotingMethod::Consensus => no_weight == BigDecimal::from_f64(0.0).unwrap(),
    };

    // Check if result already exists
    let existing_result: Option<ProposalResult> = pr::proposal_results
        .filter(pr::proposal_id.eq(proposal_id))
        .select(ProposalResult::as_select())
        .first(&mut conn)
        .ok();

    if let Some(existing) = existing_result {
        // Update existing result
        diesel::update(pr::proposal_results.filter(pr::id.eq(existing.id)))
            .set((
                pr::passed.eq(passed),
                pr::yes_weight.eq(&yes_weight),
                pr::no_weight.eq(&no_weight),
                pr::abstain_weight.eq(&abstain_weight),
                pr::total_weight.eq(&total_weight),
            ))
            .execute(&mut conn)?;
    } else {
        // Insert new result
        diesel::insert_into(pr::proposal_results)
            .values((
                pr::proposal_id.eq(proposal_id),
                pr::passed.eq(passed),
                pr::yes_weight.eq(&yes_weight),
                pr::no_weight.eq(&no_weight),
                pr::abstain_weight.eq(&abstain_weight),
                pr::total_weight.eq(&total_weight),
                pr::method_applied_version.eq("v1"),
            ))
            .execute(&mut conn)?;
    }

    // Update proposal status to Tallied
    diesel::update(p::proposals.filter(p::id.eq(proposal_id)))
        .set(p::status.eq("Tallied"))
        .execute(&mut conn)?;

    #[derive(serde::Serialize)]
    struct TallyResponse {
        success: bool,
        passed: bool,
    }

    Ok(HttpResponse::Ok().json(TallyResponse {
        success: true,
        passed,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/proposals", web::get().to(list_proposals))
        .route("/proposals", web::post().to(create_proposal))
        .route("/proposals/{id}", web::get().to(get_proposal))
        .route("/proposals/{id}/vote", web::post().to(cast_vote))
        .route("/proposals/{id}/tally", web::post().to(tally_results));
}
