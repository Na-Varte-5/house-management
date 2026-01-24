use crate::schema::{proposal_results, proposals, votes};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = proposals)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub created_by: u64,
    pub building_id: Option<u64>,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub voting_method: String,
    pub eligible_roles: String,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = proposals)]
pub struct NewProposal {
    pub title: String,
    pub description: String,
    pub building_id: Option<u64>,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub voting_method: String,
    pub eligible_roles: String,
    pub status: String,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = votes)]
pub struct Vote {
    pub id: u64,
    pub proposal_id: u64,
    pub user_id: u64,
    #[schema(value_type = String, example = "1.5")]
    pub weight_decimal: BigDecimal,
    pub choice: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = proposal_results)]
pub struct ProposalResult {
    pub id: u64,
    pub proposal_id: u64,
    pub passed: bool,
    #[schema(value_type = String, example = "5.0")]
    pub yes_weight: BigDecimal,
    #[schema(value_type = String, example = "3.0")]
    pub no_weight: BigDecimal,
    #[schema(value_type = String, example = "1.0")]
    pub abstain_weight: BigDecimal,
    #[schema(value_type = String, example = "9.0")]
    pub total_weight: BigDecimal,
    pub tallied_at: Option<chrono::NaiveDateTime>,
    pub method_applied_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VotingMethod {
    SimpleMajority,
    WeightedArea,
    PerSeat,
    Consensus,
}
impl std::fmt::Display for VotingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SimpleMajority => "SimpleMajority",
                Self::WeightedArea => "WeightedArea",
                Self::PerSeat => "PerSeat",
                Self::Consensus => "Consensus",
            }
        )
    }
}
impl std::str::FromStr for VotingMethod {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "SimpleMajority" => Self::SimpleMajority,
            "WeightedArea" => Self::WeightedArea,
            "PerSeat" => Self::PerSeat,
            "Consensus" => Self::Consensus,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}
impl std::fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Yes => "Yes",
                Self::No => "No",
                Self::Abstain => "Abstain",
            }
        )
    }
}
impl std::str::FromStr for VoteChoice {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Yes" => Self::Yes,
            "No" => Self::No,
            "Abstain" => Self::Abstain,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ProposalStatus {
    Scheduled,
    Open,
    Closed,
    Tallied,
}
impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scheduled => "Scheduled",
                Self::Open => "Open",
                Self::Closed => "Closed",
                Self::Tallied => "Tallied",
            }
        )
    }
}
impl std::str::FromStr for ProposalStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Scheduled" => Self::Scheduled,
            "Open" => Self::Open,
            "Closed" => Self::Closed,
            "Tallied" => Self::Tallied,
            _ => return Err(()),
        })
    }
}
