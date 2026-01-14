mod common;

use common::{TestServer, TestUser, create_and_login_user};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_admin_can_create_proposal() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Test Proposal",
            "description": "This is a test proposal",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner", "Admin"],
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let proposal: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(proposal["title"], "Test Proposal");
    assert_eq!(proposal["voting_method"], "SimpleMajority");
}

#[tokio::test]
async fn test_manager_can_create_proposal() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let manager = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::manager()).await;

    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(manager.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Manager Proposal",
            "description": "This is a manager proposal",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_homeowner_cannot_create_proposal() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let homeowner = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;

    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Homeowner Proposal",
            "description": "Should fail",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_proposals() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    // Create proposals
    for i in 1..=3 {
        client
            .post(&format!("{}/proposals", server.base_url))
            .bearer_auth(admin.token.as_ref().unwrap())
            .json(&serde_json::json!({
                "title": format!("Proposal {}", i),
                "description": format!("Description {}", i),
                "start_time": start_time,
                "end_time": end_time,
                "voting_method": "SimpleMajority",
                "eligible_roles": ["Homeowner"],
            }))
            .send()
            .await
            .expect("Failed to create proposal");
    }

    // List proposals
    let response = client
        .get(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let proposals: Value = response.json().await.expect("Failed to parse response");
    let proposals_array = proposals.as_array().expect("Expected array");
    assert_eq!(proposals_array.len(), 3);
}

#[tokio::test]
async fn test_eligible_user_can_vote() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;

    // Create proposal
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Vote Test",
            "description": "Test voting",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Cast vote
    let response = client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "choice": "Yes",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let vote_response: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(vote_response["success"], true);
    assert_eq!(vote_response["choice"], "Yes");
}

#[tokio::test]
async fn test_ineligible_user_cannot_vote() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let renter = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::renter()).await;

    // Create proposal only for Homeowners
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Homeowner Only",
            "description": "Only homeowners can vote",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Try to vote as renter
    let response = client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(renter.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "choice": "Yes",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_user_can_change_vote() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;

    // Create proposal
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Change Vote Test",
            "description": "Test changing vote",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Cast first vote (Yes)
    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "choice": "Yes",
        }))
        .send()
        .await
        .expect("Failed to cast first vote");

    // Change vote to No
    let response = client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "choice": "No",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let vote_response: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(vote_response["choice"], "No");

    // Verify the vote was changed
    let proposal_detail = client
        .get(&format!("{}/proposals/{}", server.base_url, proposal_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to get proposal");

    let detail: Value = proposal_detail.json().await.expect("Failed to parse response");
    assert_eq!(detail["user_vote"], "No");
    assert_eq!(detail["no_count"], 1);
    assert_eq!(detail["yes_count"], 0);
}

#[tokio::test]
async fn test_tally_simple_majority_passes() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create multiple homeowners for voting
    let homeowner1 = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;
    let mut homeowner2 = TestUser::homeowner();
    homeowner2.email = "homeowner2@test.com".to_string();
    let homeowner2 = create_and_login_user(&server.pool, &client, &server.base_url, homeowner2).await;
    let mut homeowner3 = TestUser::homeowner();
    homeowner3.email = "homeowner3@test.com".to_string();
    let homeowner3 = create_and_login_user(&server.pool, &client, &server.base_url, homeowner3).await;

    // Create proposal
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Tally Test",
            "description": "Test tallying",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Cast votes: 2 Yes, 1 No
    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner1.token.as_ref().unwrap())
        .json(&serde_json::json!({"choice": "Yes"}))
        .send()
        .await
        .expect("Failed to vote");

    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner2.token.as_ref().unwrap())
        .json(&serde_json::json!({"choice": "Yes"}))
        .send()
        .await
        .expect("Failed to vote");

    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner3.token.as_ref().unwrap())
        .json(&serde_json::json!({"choice": "No"}))
        .send()
        .await
        .expect("Failed to vote");

    // Tally results
    let response = client
        .post(&format!("{}/proposals/{}/tally", server.base_url, proposal_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let tally: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(tally["passed"], true);
}

#[tokio::test]
async fn test_tally_simple_majority_fails() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    let homeowner1 = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;
    let mut homeowner2 = TestUser::homeowner();
    homeowner2.email = "homeowner2@test.com".to_string();
    let homeowner2 = create_and_login_user(&server.pool, &client, &server.base_url, homeowner2).await;

    // Create proposal
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Fail Test",
            "description": "Should fail",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Cast votes: 1 Yes, 2 No
    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner1.token.as_ref().unwrap())
        .json(&serde_json::json!({"choice": "Yes"}))
        .send()
        .await
        .expect("Failed to vote");

    client
        .post(&format!("{}/proposals/{}/vote", server.base_url, proposal_id))
        .bearer_auth(homeowner2.token.as_ref().unwrap())
        .json(&serde_json::json!({"choice": "No"}))
        .send()
        .await
        .expect("Failed to vote");

    // Tally results
    let response = client
        .post(&format!("{}/proposals/{}/tally", server.base_url, proposal_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let tally: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(tally["passed"], false);
}

#[tokio::test]
async fn test_homeowner_cannot_tally() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(&server.pool, &client, &server.base_url, TestUser::homeowner()).await;

    // Create proposal
    let start_time = chrono::Local::now().naive_local().format("%Y-%m-%dT%H:%M").to_string();
    let end_time = (chrono::Local::now() + chrono::Duration::days(7)).naive_local().format("%Y-%m-%dT%H:%M").to_string();

    let proposal_response = client
        .post(&format!("{}/proposals", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "title": "Tally Permission Test",
            "description": "Test tallying permission",
            "start_time": start_time,
            "end_time": end_time,
            "voting_method": "SimpleMajority",
            "eligible_roles": ["Homeowner"],
        }))
        .send()
        .await
        .expect("Failed to create proposal");

    let proposal: Value = proposal_response.json().await.expect("Failed to parse response");
    let proposal_id = proposal["id"].as_u64().expect("No proposal ID");

    // Try to tally as homeowner
    let response = client
        .post(&format!("{}/proposals/{}/tally", server.base_url, proposal_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
